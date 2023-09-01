use actix_web::{get, web, Responder};
use mockall_double::double;
use std::{pin::Pin, time::Duration};

use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use futures::{channel::mpsc, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::log::{info, warn};

#[double]
use crate::database::Database;
use crate::{
    app::PlantingLifeApp,
    domain::*,
    hydrator::{self, Hydrator},
    selector::Selector,
};

#[derive(Serialize, Deserialize, Debug)]
struct PlantsStreamRequest {
    zip: String,
    shade: Shade,
    moisture: Moisture,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlantsByNameRequest {
    name: String,
}

pub struct PlantController {
    pub db: &'static Database,
    pub hydrator: Hydrator,
    pub selector: Selector,
}

impl PlantController {
    pub fn new(db: &'static Database, hydrator: Hydrator, selector: Selector) -> Self {
        Self {
            db,
            hydrator,
            selector,
        }
    }

    async fn search_by_name(
        &'static self,
        payload: PlantsByNameRequest,
    ) -> Result<impl Responder, actix_web::Error> {
        // Don't search for very short strings, force more text before searching
        if payload.name.len() < 3 {
            return Ok(actix_web::HttpResponse::Ok().body("[]"));
        }

        let plants = self.db.find_plants_by_word_prefix(&payload.name).await;
        Ok(actix_web::HttpResponse::Ok().json(plants))
    }

    async fn stream(
        &'static self,
        payload: PlantsStreamRequest,
    ) -> Result<impl Responder, actix_web::Error> {
        info!("{payload:?}");

        let valid_zip = self.get_closest_valid_zip(&payload.zip).await?;
        let valid_payload = PlantsStreamRequest {
            zip: valid_zip,
            ..payload
        };
        drop(payload); // Don't use the unvalidated payload by mistake

        let (frontend_sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(10);

        // The real work is done in a new thread so the connection to the front end can stay open.
        actix_web::rt::spawn(async move {
            let plant_stream = self
                .selector
                .stream_plants(
                    &valid_payload.zip,
                    &valid_payload.moisture,
                    &valid_payload.shade,
                )
                .await;

            match plant_stream {
                Ok(plant_stream) => {
                    self.hydrate_and_send_plants(
                        Some(valid_payload),
                        plant_stream.stream,
                        &frontend_sender,
                        plant_stream.from_db,
                    )
                    .await
                }
                Err(e) => {
                    warn!("error getting plant stream: {e}");
                    send_event(&frontend_sender, "error", "").await
                }
            };

            send_event(&frontend_sender, "close", "").await;
        });

        Ok(stream
            .with_keep_alive(Duration::from_secs(1))
            .customize()
            .insert_header(("X-Accel-Buffering", "no")))
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

    async fn hydrate_and_send_plants(
        &'static self,
        payload: Option<PlantsStreamRequest>,
        plant_stream: Pin<Box<dyn Stream<Item = Plant> + Send>>,
        frontend_sender: &Sender,
        plants_from_db: bool,
    ) {
        let (mut plant_sender, mut plant_receiver) = mpsc::unbounded();

        actix_web::rt::spawn(async move {
            self.hydrator
                .hydrate_plants(plant_stream, &mut plant_sender)
                .await;
        });

        let mut all_plants = vec![];
        let mut saved_plants = vec![];
        while let Some(hydrated_plant) = plant_receiver.next().await {
            if hydrated_plant.plant.scientific == hydrator::ALL_PLANTS_HYDRATING_MARKER {
                send_event(frontend_sender, "allPlantsLoaded", "").await;

                continue;
            }

            send_plant(frontend_sender, &hydrated_plant.plant).await;

            if hydrated_plant.plant.done_loading {
                all_plants.push(hydrated_plant.plant.clone());

                if hydrated_plant.updated {
                    //plants_to_save.push(hydrated_plant.plant.clone());
                    match self.db.save_plant(&hydrated_plant.plant).await {
                        Ok(plant) => saved_plants.push(plant.clone()),
                        Err(e) => warn!(
                            "Failed to save {} due to {e}",
                            hydrated_plant.plant.scientific
                        ),
                    }
                }
            }
        }

        // We only need to cache the query results if these results aren't from the database
        // When they are from the database, we know its already there.
        if let Some(payload) = payload {
            if !plants_from_db {
                self.db
                    .save_query_results(
                        &payload.zip,
                        &payload.moisture,
                        &payload.shade,
                        all_plants,
                        saved_plants,
                    )
                    .await;
            }
        }
    }

    /// Streams one plant back by scientific name.  Uses a stream because it may
    /// need to be populated still and it should load incrementally.  Uses
    /// scientific name rather than id because id makes it trivial to download
    /// the entire database :)
    async fn stream_by_scientific_name(
        &'static self,
        id: String,
    ) -> Result<impl Responder, actix_web::Error> {
        info!("stream_by_scientific_name {id}");

        let (frontend_sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(10);

        // The real work is done in a new thread so the connection to the front end can stay open.
        actix_web::rt::spawn(async move {
            let plant = self.db.get_plant_by_scientific_name(&id).await;
            if plant.is_none() {
                warn!("Couldn't find plant with id: {id}");
                return;
            }

            // Create a stream to interface nicely with hydrate/send function
            // Unwrap is safe due to is_none check above.
            let stream = Box::pin(futures::stream::iter(vec![plant.unwrap()]));

            self.hydrate_and_send_plants(None, stream, &frontend_sender, true)
                .await;

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

#[get("/plants/stream")]
async fn plants_stream_handler(
    web::Query(payload): web::Query<PlantsStreamRequest>,
    app: web::Data<&'static PlantingLifeApp>,
) -> Result<impl Responder, actix_web::Error> {
    app.plant_controller.stream(payload).await
}

#[get("/plants/stream/{scientific_name}")]
async fn plants_stream_by_scientific_name_handler(
    id: web::Path<String>,
    app: web::Data<&'static PlantingLifeApp>,
) -> Result<impl Responder, actix_web::Error> {
    app.plant_controller
        .stream_by_scientific_name(id.to_string())
        .await
}

#[get("/plants")]
async fn plants_by_name_handler(
    web::Query(payload): web::Query<PlantsByNameRequest>,
    app: web::Data<&'static PlantingLifeApp>,
) -> Result<impl Responder, actix_web::Error> {
    app.plant_controller.search_by_name(payload).await
}
