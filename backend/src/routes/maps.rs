use actix_web::{get, web, Responder};
use crate::services::maps::MapsService;

#[get("/maps/api-key")]
async fn maps_api_key_handler(
    service: web::Data<MapsService>,
) -> impl Responder {
    match service.get_api_key().await {
        Ok(api_key) => actix_web::HttpResponse::Ok().body(api_key),
        Err(e) if e.to_string() == "Rate limit exceeded" => {
            actix_web::HttpResponse::ServiceUnavailable().finish()
        }
        Err(_) => actix_web::HttpResponse::NotFound().finish(),
    }
}
