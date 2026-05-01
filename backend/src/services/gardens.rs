use std::sync::Arc;

use mockall_double::double;
use serde::{Deserialize, Serialize};
use tracing::log::{info, warn};

#[double]
use crate::database::Database;
use crate::{
    domain::{Garden, Moisture, Plant, Shade},
    highlights::Highlights,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGardenCommand {
    pub plant_ids: Vec<usize>,
    pub zipcode: String,
    pub moisture: Moisture,
    pub shade: Shade,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateGardenCommand {
    pub write_id: String,
    pub plant_ids: Vec<usize>,
    pub name: String,
}

#[derive(Clone)]
pub struct GardenService {
    pub db: Arc<Database>,
    pub highlights: Arc<Highlights>,
}

impl GardenService {
    pub fn new(db: Arc<Database>, highlights: Arc<Highlights>) -> Self {
        Self { db, highlights }
    }

    pub async fn create(&self, cmd: CreateGardenCommand) -> anyhow::Result<Garden> {
        info!("{cmd:?}");

        let region_name = self
            .db
            .get_region_name_by_zip(&cmd.zipcode)
            .await
            .unwrap_or_else(|| format!("Zipcode {}", cmd.zipcode));

        let name = cmd
            .name
            .unwrap_or_else(|| format!("Native Garden near {region_name}"));

        let mut garden = Garden::empty(name, cmd.zipcode, cmd.shade, cmd.moisture);

        let (read_id, write_id) = self.db.save_new_garden(&garden, cmd.plant_ids).await?;
        garden.read_id = Some(read_id);
        garden.write_id = Some(write_id);

        Ok(garden)
    }

    pub async fn update(&self, cmd: UpdateGardenCommand) -> anyhow::Result<()> {
        info!("{cmd:?}");

        self.db
            .save_existing_garden(&cmd.write_id, &cmd.name, cmd.plant_ids)
            .await
    }

    pub async fn read(&self, id: &str) -> Option<Garden> {
        info!("GardensGetRequest id: {id}");

        // Fetch the garden, then populate the highlights on each plant
        let garden = self.db.get_garden(id).await.map(|g| Garden {
            plants: g
                .plants
                .into_iter()
                .map(|p| Plant {
                    highlights: self.highlights.generate(&p),
                    ..p
                })
                .collect(),
            ..g
        });

        garden
    }

    pub async fn list(&self, require_precise_location: bool) -> Vec<Garden> {
        info!("GardensListRequest: {require_precise_location:?}");

        // Nobody needs to request all gardens with no filters.
        if !require_precise_location {
            warn!("Attempt to list all gardens with no filters, returning nothing");
            return vec![];
        }

        self.db.list_gardens(require_precise_location).await
    }
}
