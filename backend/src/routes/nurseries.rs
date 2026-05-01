use actix_web::{get, web, Responder};
use serde::{Deserialize, Serialize};
use crate::services::nurseries::NurseriesService;

#[derive(Deserialize, Serialize, Debug)]
struct NurseriesRequest {
    zip: String,
}

#[get("/nurseries")]
async fn fetch_nurseries_handler(
    web::Query(payload): web::Query<NurseriesRequest>,
    service: web::Data<NurseriesService>,
) -> impl Responder {
    let nurseries = service.list(payload.zip).await;
    actix_web::HttpResponse::Ok().json(nurseries)
}
