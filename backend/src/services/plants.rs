use mockall_double::double;
use tracing::log::{info, warn};

#[double]
use crate::database::Database;
use crate::{domain::*, highlights::Highlights};

pub async fn find_plants(
    db: &Database,
    highlights: &Highlights,
    cmd: PlantSearchCommand,
) -> anyhow::Result<Vec<Plant>> {
    info!("{cmd:?}");

    let plants = match cmd {
        PlantSearchCommand {
            name: None,
            zip: Some(zip),
            moisture: Some(moisture),
            shade: Some(shade),
            habit,
        } => {
            let zip = get_closest_valid_zip(db, &zip).await?;
            db.lookup_query_results(&zip, &moisture, &shade, &habit)
                .await
        }
        PlantSearchCommand {
            name: Some(name),
            zip: None,
            moisture: None,
            shade: None,
            habit: None,
        } => db.find_plants_by_word_prefix(&name).await,
        _ => {
            return Err(anyhow::anyhow!(
                "either name OR zip/shade/moisture are required"
            ));
        }
    };

    Ok(populate_highlights(highlights, plants))
}

pub async fn find_plant(db: &Database, highlights: &Highlights, id: usize) -> Option<Plant> {
    info!("find_plant {id}");

    let plant = db.get_plant_by_id(id).await?;
    Some(Plant {
        highlights: highlights.generate(&plant),
        ..plant
    })
}

async fn get_closest_valid_zip(db: &Database, zip: &str) -> anyhow::Result<String> {
    let valid_zip = db.lookup_closest_valid_zip(zip).await.map_err(|e| {
        warn!("Cannot find valid zipcode: {e}");
        anyhow::anyhow!("invalid zipcode")
    })?;

    if valid_zip != zip {
        info!("Adjusted unknown zip {} to {valid_zip}", zip);
    }

    Ok(valid_zip)
}

fn populate_highlights(highlights: &Highlights, plants: Vec<Plant>) -> Vec<Plant> {
    plants
        .into_iter()
        .map(|mut p| {
            p.highlights = highlights.generate(&p);
            p
        })
        .collect()
}

#[derive(Debug)]
pub struct PlantSearchCommand {
    pub name: Option<String>,
    pub zip: Option<String>,
    pub shade: Option<Shade>,
    pub moisture: Option<Moisture>,
    pub habit: Option<Habit>,
}
