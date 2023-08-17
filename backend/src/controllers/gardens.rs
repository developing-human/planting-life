use actix_web::{get, post, put, web, Responder};
use mockall_double::double;
use serde::{Deserialize, Serialize};
use tracing::log::{info, warn};

#[double]
use crate::database::Database;
use crate::{
    app::PlantingLifeApp,
    domain::{Garden, Moisture, Plant, Shade},
    highlights::Highlights,
};

#[derive(Serialize, Deserialize, Debug)]
struct GardensPostRequest {
    plant_ids: Vec<usize>,
    name: String,
    zipcode: String,
    moisture: Moisture,
    shade: Shade,
}

#[derive(Serialize, Deserialize, Debug)]
struct GardensPutRequest {
    plant_ids: Vec<usize>,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GardensPostResponse {
    read_id: String,
    write_id: String,
}

pub struct GardensController {
    pub db: &'static Database,
    pub highlights: Highlights,
}

impl GardensController {
    pub fn new(db: &'static Database, highlights: Highlights) -> Self {
        Self { db, highlights }
    }

    async fn create(&self, payload: GardensPostRequest) -> impl Responder {
        info!("{payload:?}");

        let garden = Garden::empty(
            payload.name,
            payload.zipcode,
            payload.shade,
            payload.moisture,
        );

        let response = match self.db.save_new_garden(&garden, payload.plant_ids).await {
            Ok((read_id, write_id)) => GardensPostResponse { read_id, write_id },
            Err(e) => {
                warn!("Error saving garden: {e}");
                return actix_web::HttpResponse::InternalServerError()
                    .body("Could not save garden");
            }
        };

        actix_web::HttpResponse::Ok().json(response)
    }

    async fn update(&self, write_id: &str, payload: GardensPutRequest) -> impl Responder {
        info!("{payload:?}");

        match self
            .db
            .save_existing_garden(write_id, &payload.name, payload.plant_ids)
            .await
        {
            Ok(()) => actix_web::HttpResponse::Ok().body(""),
            Err(e) => {
                warn!("Error saving garden: {e}");
                actix_web::HttpResponse::InternalServerError().body("Could not save garden")
            }
        }
    }

    async fn read(&self, id: &str) -> impl Responder {
        // Fetch the garden, then populate the highlights on each plant
        let garden = self.db.get_garden(id).await.map(|g| Garden {
            plants: g
                .plants
                .into_iter()
                .map(|p| Plant {
                    highlights: self.highlights.generate(&p),
                    ..p
                })
                .collect(),
            ..g
        });

        match garden {
            Some(garden) => actix_web::HttpResponse::Ok().json(garden),
            None => actix_web::HttpResponse::NotFound().body(""),
        }
    }
}

#[get("/gardens/{id}")]
async fn read_garden_handler(
    id: web::Path<String>,
    app: web::Data<&'static PlantingLifeApp>,
) -> impl Responder {
    app.gardens_controller.read(&id).await
}

#[post("/gardens")]
async fn create_garden_handler(
    web::Json(payload): web::Json<GardensPostRequest>,
    app: web::Data<&'static PlantingLifeApp>,
) -> impl Responder {
    app.gardens_controller.create(payload).await
}

#[put("/gardens/{id}")]
async fn update_garden_handler(
    write_id: web::Path<String>,
    web::Json(payload): web::Json<GardensPutRequest>,
    app: web::Data<&'static PlantingLifeApp>,
) -> impl Responder {
    println!("put");
    app.gardens_controller.update(&write_id, payload).await
}
