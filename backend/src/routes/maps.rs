use crate::{services, state::AppState};
use actix_web::{get, web, Responder};

#[get("/maps/api-key")]
async fn maps_api_key_handler(state: web::Data<AppState>) -> impl Responder {
    match services::maps::get_api_key(&state.db).await {
        Ok(api_key) => actix_web::HttpResponse::Ok().body(api_key),
        Err(e) if e.to_string() == "Rate limit exceeded" => {
            actix_web::HttpResponse::ServiceUnavailable().finish()
        }
        Err(_) => actix_web::HttpResponse::NotFound().finish(),
    }
}
