use crate::{
    domain::{Habit, Moisture, Shade},
    services::plants::{PlantSearchCommand, PlantService},
};
use actix_web::{get, web, Responder};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct PlantSearchRequest {
    pub name: Option<String>,
    pub zip: Option<String>,
    pub shade: Option<Shade>,
    pub moisture: Option<Moisture>,
    pub habit: Option<Habit>,
}

impl From<PlantSearchRequest> for PlantSearchCommand {
    fn from(value: PlantSearchRequest) -> Self {
        Self {
            name: value.name,
            zip: value.zip,
            shade: value.shade,
            moisture: value.moisture,
            habit: value.habit,
        }
    }
}

#[get("/plants/{id}")]
async fn find_plant_handler(
    id: web::Path<usize>,
    service: web::Data<PlantService>,
) -> impl Responder {
    match service.find_plant(*id).await {
        Some(plant) => actix_web::HttpResponse::Ok().json(plant),
        None => actix_web::HttpResponse::NotFound().body("plant not found"),
    }
}

#[get("/plants")]
async fn find_plants_handler(
    web::Query(payload): web::Query<PlantSearchRequest>,
    service: web::Data<PlantService>,
) -> impl Responder {
    match service.find_plants(payload.into()).await {
        Ok(plants) => actix_web::HttpResponse::Ok().json(plants),
        Err(e) => actix_web::HttpResponse::BadRequest().body(e.to_string()),
    }
}
