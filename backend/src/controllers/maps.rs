use std::{env, sync::Arc};

use actix_web::{get, web, Responder};
use mockall_double::double;
use tracing::log::info;

use crate::app::PlantingLifeApp;

#[double]
use crate::database::Database;

pub struct MapsController {
    pub db: Arc<Database>,
}

impl MapsController {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    async fn get_api_key(&self) -> impl Responder {
        info!("fetching maps api key");

        let uri = "/maps/api-key";
        self.db.update_request_count(uri).await;
        let monthly_count = self.db.get_monthly_request_count(uri).await;
        info!("monthly_count is {monthly_count}");

        if monthly_count >= 10_000 {
            return actix_web::HttpResponse::ServiceUnavailable().finish();
        }

        if let Ok(api_key) = env::var("PLANTING_LIFE_MAPS_API_KEY") {
            actix_web::HttpResponse::Ok().body(api_key)
        } else {
            actix_web::HttpResponse::NotFound().finish()
        }
    }
}

#[get("/maps/api-key")]
async fn maps_api_key_handler(app: web::Data<&'static PlantingLifeApp>) -> impl Responder {
    app.maps_controller.get_api_key().await
}
