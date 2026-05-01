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

pub async fn create(db: &Database, cmd: CreateGardenCommand) -> anyhow::Result<Garden> {
    info!("{cmd:?}");

    let region_name = db
        .get_region_name_by_zip(&cmd.zipcode)
        .await
        .unwrap_or_else(|| format!("Zipcode {}", cmd.zipcode));

    let name = cmd
        .name
        .unwrap_or_else(|| format!("Native Garden near {region_name}"));

    let mut garden = Garden::empty(name, cmd.zipcode, cmd.shade, cmd.moisture);

    let (read_id, write_id) = db.save_new_garden(&garden, cmd.plant_ids).await?;
    garden.read_id = Some(read_id);
    garden.write_id = Some(write_id);

    Ok(garden)
}

pub async fn update(db: &Database, cmd: UpdateGardenCommand) -> anyhow::Result<()> {
    info!("{cmd:?}");

    db.save_existing_garden(&cmd.write_id, &cmd.name, cmd.plant_ids)
        .await
}

pub async fn read(db: &Database, highlights: &Highlights, id: &str) -> Option<Garden> {
    info!("GardensGetRequest id: {id}");

    // Fetch the garden, then populate the highlights on each plant
    db.get_garden(id).await.map(|g| Garden {
        plants: g
            .plants
            .into_iter()
            .map(|p| Plant {
                highlights: highlights.generate(&p),
                ..p
            })
            .collect(),
        ..g
    })
}

pub async fn list(db: &Database, require_precise_location: bool) -> Vec<Garden> {
    info!("GardensListRequest: {require_precise_location:?}");

    // Nobody needs to request all gardens with no filters.
    if !require_precise_location {
        warn!("Attempt to list all gardens with no filters, returning nothing");
        return vec![];
    }

    db.list_gardens(require_precise_location).await
}
