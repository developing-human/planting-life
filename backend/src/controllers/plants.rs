use actix_web::{get, web, HttpResponse, Responder};
use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use mockall_double::double;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::log::{info, warn};

#[double]
use crate::database::Database;
use crate::{app::PlantingLifeApp, domain::*, highlights::Highlights};

#[derive(Serialize, Deserialize, Debug)]
struct PlantsStreamRequest {
    zip: String,
    shade: Shade,
    moisture: Moisture,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlantSearchRequest {
    name: Option<String>,
    zip: Option<String>,
    shade: Option<Shade>,
    moisture: Option<Moisture>,
}

pub struct PlantController {
    pub db: &'static Database,
    pub highlights: &'static Highlights,
}

impl PlantController {
    pub fn new(db: &'static Database, highlights: &'static Highlights) -> Self {
        Self { db, highlights }
    }

    async fn find_plants(&'static self, payload: PlantSearchRequest) -> impl Responder {
        // find_plants can be used in two ways:
        // 1. searching by zip/shade/moisture
        // 2. search by partial name match

        let plants = match payload {
            PlantSearchRequest {
                name: None,
                zip: Some(zip),
                moisture: Some(moisture),
                shade: Some(shade),
            } => {
                // adjust zip codes which aren't in the database to the closest
                // one that is, because not every zip is in the db
                let zip = self.get_closest_valid_zip(&zip).await.unwrap_or(zip);

                self.db.lookup_query_results(&zip, &moisture, &shade).await
            }
            PlantSearchRequest {
                name: Some(name),
                zip: None,
                moisture: None,
                shade: None,
            } => self.db.find_plants_by_word_prefix(&name).await,
            _ => {
                return HttpResponse::BadRequest()
                    .body("either name OR zip/shade/moisture are required")
            }
        };

        // The plants which come back from the database don't have highlights,
        // as those are derived from ratings.  Populate those, and mark it as
        // done loading (at least until that field goes away).
        let plants: Vec<Plant> = plants
            .into_iter()
            .map(|p| Plant {
                highlights: self.highlights.generate(&p),
                ..p
            })
            .collect();

        actix_web::HttpResponse::Ok().json(plants)
    }

    async fn find_plant(&'static self, id: usize) -> impl Responder {
        info!("find_plant {id}");

        if let Some(mut plant) = self.db.get_plant_by_id(id).await {
            plant.highlights = self.highlights.generate(&plant);
            actix_web::HttpResponse::Ok().json(plant)
        } else {
            actix_web::HttpResponse::NotFound().body("plant not found")
        }
    }

    async fn get_closest_valid_zip(&self, zip: &str) -> Result<String, actix_web::Error> {
        let valid_zip = self.db.lookup_closest_valid_zip(zip).await.map_err(|e| {
            warn!("Cannot find valid zipcode: {e}");
            actix_web::error::ErrorBadRequest("invalid zipcode")
        })?;

        if valid_zip != zip {
            info!("Adjusted unknown zip {} to {valid_zip}", zip);
        }

        Ok(valid_zip)
    }

    //TODO: Remove once browser caches are likely clear, this is legacy support
    async fn stream(
        &'static self,
        payload: PlantsStreamRequest,
    ) -> Result<impl Responder, actix_web::Error> {
        info!("{payload:?}");

        let valid_zip = self.get_closest_valid_zip(&payload.zip).await?;
        let plants = self
            .db
            .lookup_query_results(&valid_zip, &payload.moisture, &payload.shade)
            .await;

        let plants: Vec<Plant> = plants
            .into_iter()
            .map(|p| Plant {
                highlights: self.highlights.generate(&p),
                ..p
            })
            .collect();

        let (frontend_sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(11);

        // The real work is done in a new thread so the connection to the front end can stay open.
        actix_web::rt::spawn(async move {
            for plant in plants {
                send_plant(&frontend_sender, &plant).await;
            }

            send_event(&frontend_sender, "close", "").await;
        });

        Ok(stream
            .with_keep_alive(Duration::from_secs(1))
            .customize()
            .insert_header(("X-Accel-Buffering", "no")))
    }

    //TODO: Remove once browser caches are likely clear, this is legacy support
    /// Streams one plant back by scientific name.
    async fn stream_by_scientific_name(
        &'static self,
        name: String,
    ) -> Result<impl Responder, actix_web::Error> {
        info!("stream_by_scientific_name {name}");

        let (frontend_sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(10);

        // The real work is done in a new thread so the connection to the front end can stay open.
        actix_web::rt::spawn(async move {
            let plant = self.db.get_plant_by_scientific_name(&name).await;
            if plant.is_none() {
                warn!("Couldn't find plant with name: {name}");
                return;
            }
            let mut plant = plant.unwrap();
            plant.highlights = self.highlights.generate(&plant);

            send_plant(&frontend_sender, &plant).await;
            send_event(&frontend_sender, "close", "").await;
        });

        Ok(stream
            .with_keep_alive(Duration::from_secs(1))
            .customize()
            .insert_header(("X-Accel-Buffering", "no")))
    }
}

async fn send_plant(sender: &Sender, plant: &Plant) {
    let json = serde_json::to_string(&plant).expect("plant should serialize");

    send_event(sender, "plant", &json).await;
}

async fn send_event(sender: &Sender, event: &str, message: &str) {
    let data = sse::Data::new(message).event(event);

    match sender.send(data).await {
        Ok(_) => {}
        Err(_) => warn!("Error sending [{}] with message [{}]", event, message),
    }
}

#[get("/plants/{id}")]
async fn find_plant_handler(
    id: web::Path<usize>,
    app: web::Data<&'static PlantingLifeApp>,
) -> impl Responder {
    app.plant_controller.find_plant(*id).await
}

#[get("/plants")]
async fn find_plants_handler(
    web::Query(payload): web::Query<PlantSearchRequest>,
    app: web::Data<&'static PlantingLifeApp>,
) -> impl Responder {
    app.plant_controller.find_plants(payload).await
}

//TODO: Remove after frontend cache seems clear
#[get("/plants/stream")]
async fn plants_stream_handler(
    web::Query(payload): web::Query<PlantsStreamRequest>,
    app: web::Data<&'static PlantingLifeApp>,
) -> Result<impl Responder, actix_web::Error> {
    println!("in stream");
    app.plant_controller.stream(payload).await
}

//TODO: Remove after frontend cache seems clear
#[get("/plants/stream/{scientific_name}")]
async fn plants_stream_by_scientific_name_handler(
    id: web::Path<String>,
    app: web::Data<&'static PlantingLifeApp>,
) -> Result<impl Responder, actix_web::Error> {
    app.plant_controller
        .stream_by_scientific_name(id.to_string())
        .await
}
