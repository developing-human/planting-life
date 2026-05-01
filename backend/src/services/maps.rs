use std::{env, sync::Arc};
use mockall_double::double;
use tracing::log::info;

#[double]
use crate::database::Database;

#[derive(Clone)]
pub struct MapsService {
    pub db: Arc<Database>,
}

impl MapsService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_api_key(&self) -> anyhow::Result<String> {
        info!("fetching maps api key");

        let uri = "/maps/api-key";
        self.db.update_request_count(uri).await;
        let monthly_count = self.db.get_monthly_request_count(uri).await;
        info!("monthly_count is {monthly_count}");

        if monthly_count >= 10_000 {
            return Err(anyhow::anyhow!("Rate limit exceeded"));
        }

        let api_key = env::var("PLANTING_LIFE_MAPS_API_KEY")?;
        Ok(api_key)
    }
}
