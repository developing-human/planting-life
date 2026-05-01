use crate::{services, state::AppState};
use actix_web::{get, web, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct NurseriesRequest {
    zip: String,
}

#[get("/nurseries")]
async fn fetch_nurseries_handler(
    web::Query(payload): web::Query<NurseriesRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let nurseries = services::nurseries::list(&state.db, payload.zip).await;
    actix_web::HttpResponse::Ok().json(nurseries)
}
