use actix_web::{get, web, HttpResponse, Responder};
use mockall_double::double;
use serde::{Deserialize, Serialize};
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
                done_loading: true,
                ..p
            })
            .collect();

        actix_web::HttpResponse::Ok().json(plants)
    }

    async fn find_plant(&'static self, id: usize) -> impl Responder {
        info!("find_plant {id}");

        if let Some(mut plant) = self.db.get_plant_by_id(id).await {
            plant.done_loading = true;
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
