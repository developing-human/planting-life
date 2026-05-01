use actix_web::{get, post, put, web, Responder};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    domain::{Garden, Moisture, Shade},
    services::{
        self,
        gardens::{CreateGardenCommand, UpdateGardenCommand},
    },
    state::AppState,
};

#[derive(Serialize, Deserialize, Debug)]
struct GardensPostRequest {
    plant_ids: Vec<usize>,
    zipcode: String,
    moisture: Moisture,
    shade: Shade,
    name: Option<String>,
}

impl From<GardensPostRequest> for CreateGardenCommand {
    fn from(value: GardensPostRequest) -> Self {
        Self {
            plant_ids: value.plant_ids,
            zipcode: value.zipcode,
            moisture: value.moisture,
            shade: value.shade,
            name: value.name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GardensPostResponse {
    read_id: String,
    write_id: String,
    name: String,
}

impl From<Garden> for GardensPostResponse {
    fn from(garden: Garden) -> Self {
        Self {
            read_id: garden.read_id.unwrap_or_default(),
            write_id: garden.write_id.unwrap_or_default(),
            name: garden.name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GardensPutRequest {
    plant_ids: Vec<usize>,
    name: String,
}
impl From<(String, GardensPutRequest)> for UpdateGardenCommand {
    fn from((write_id, payload): (String, GardensPutRequest)) -> Self {
        Self {
            write_id,
            plant_ids: payload.plant_ids,
            name: payload.name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GardensListRequest {
    #[serde(default)]
    require_precise_location: bool,
}

#[get("/gardens/{id}")]
async fn read_garden_handler(id: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    match services::gardens::read(&state.db, &state.highlights, &id).await {
        Some(garden) => actix_web::HttpResponse::Ok().json(garden),
        None => actix_web::HttpResponse::NotFound().body(""),
    }
}

#[get("/gardens")]
async fn list_garden_handler(
    web::Query(payload): web::Query<GardensListRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let gardens = services::gardens::list(&state.db, payload.require_precise_location).await;

    actix_web::HttpResponse::Ok().json(gardens)
}

#[post("/gardens")]
async fn create_garden_handler(
    web::Json(payload): web::Json<GardensPostRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let garden = services::gardens::create(&state.db, payload.into()).await;

    match garden {
        Ok(garden) => {
            let response: GardensPostResponse = garden.into();
            actix_web::HttpResponse::Ok().json(response)
        }
        Err(e) => {
            warn!("Error saving garden: {e}");
            actix_web::HttpResponse::InternalServerError().body("Could not save garden")
        }
    }
}

#[put("/gardens/{id}")]
async fn update_garden_handler(
    write_id: web::Path<String>,
    web::Json(payload): web::Json<GardensPutRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let command = (write_id.into_inner(), payload).into();
    match services::gardens::update(&state.db, command).await {
        Ok(()) => actix_web::HttpResponse::Ok().body(""),
        Err(e) => {
            warn!("Error saving garden: {e}");
            actix_web::HttpResponse::InternalServerError().body("Could not save garden")
        }
    }
}
