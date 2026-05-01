use mockall_double::double;
use std::sync::Arc;
use tracing::log::info;

#[double]
use crate::database::Database;
use crate::domain::Nursery;

#[derive(Clone)]
pub struct NurseriesService {
    pub db: Arc<Database>,
}

impl NurseriesService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn list(&self, zip: String) -> Vec<Nursery> {
        info!("listing nurseries for zip: {zip}");

        let mut nurseries = self.db.find_nurseries(&zip).await;

        nurseries.truncate(10);

        for nursery in &mut nurseries {
            if nursery.map_url.is_none() {
                let zip_padded = format!("{:05}", nursery.zip);
                let query = format!("{} near {}", nursery.name, zip_padded).replace(' ', "+");
                let url = format!("https://www.google.com/maps/search/?api=1&query={query}");
                nursery.map_url = Some(url);
            }
        }

        nurseries
    }
}
