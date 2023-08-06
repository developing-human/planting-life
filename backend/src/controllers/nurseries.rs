use actix_web::{get, web, Responder};
use mockall_double::double;
use serde::{Deserialize, Serialize};
use tracing::log::info;

use crate::app::PlantingLifeApp;
#[double]
use crate::database::Database;

#[derive(Serialize, Deserialize, Debug)]
struct NurseriesRequest {
    zip: String,
}

pub struct NurseriesController {
    pub db: &'static Database,
}

impl NurseriesController {
    pub fn new(db: &'static Database) -> Self {
        Self { db }
    }

    async fn list(&self, payload: NurseriesRequest) -> impl Responder {
        info!("{payload:?}");

        // Purposefully NOT adjusting zipcode for nursery search.
        // This degrades nicely and all the distances would be incorrect
        // if the zipcode isn't known.

        let mut nurseries = self.db.find_nurseries(&payload.zip).await;

        // Some areas have 20+ nurseries and it looks ridiculous, set a limit
        nurseries.truncate(10);

        for nursery in &mut nurseries {
            if nursery.map_url.is_none() {
                // Pad the zip code to five digits, using zeros.
                let zip = format!("{:05}", nursery.zip);

                let query = format!("{} near {}", nursery.name, zip);
                let query = query.replace(' ', "+");
                let url = format!("https://www.google.com/maps/search/?api=1&query={query}");

                nursery.map_url = Some(url);
            }
        }

        actix_web::HttpResponse::Ok().json(nurseries)
    }
}

#[get("/nurseries")]
async fn fetch_nurseries_handler(
    web::Query(payload): web::Query<NurseriesRequest>,
    app: web::Data<&'static PlantingLifeApp>,
) -> impl Responder {
    app.nursery_controller.list(payload).await
}
