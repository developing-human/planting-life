use mockall_double::double;
use std::sync::Arc;
use tracing::log::{info, warn};

#[double]
use crate::database::Database;
use crate::{domain::*, highlights::Highlights};

#[derive(Clone)]
pub struct PlantService {
    pub db: Arc<Database>,
    pub highlights: Arc<Highlights>,
}

impl PlantService {
    pub fn new(db: Arc<Database>, highlights: Arc<Highlights>) -> Self {
        Self { db, highlights }
    }

    pub async fn find_plants(&self, cmd: PlantSearchCommand) -> anyhow::Result<Vec<Plant>> {
        info!("{cmd:?}");

        let plants = match cmd {
            PlantSearchCommand {
                name: None,
                zip: Some(zip),
                moisture: Some(moisture),
                shade: Some(shade),
                habit,
            } => {
                let zip = self.get_closest_valid_zip(&zip).await?;
                self.db
                    .lookup_query_results(&zip, &moisture, &shade, &habit)
                    .await
            }
            PlantSearchCommand {
                name: Some(name),
                zip: None,
                moisture: None,
                shade: None,
                habit: None,
            } => self.db.find_plants_by_word_prefix(&name).await,
            _ => {
                return Err(anyhow::anyhow!(
                    "either name OR zip/shade/moisture are required"
                ));
            }
        };

        Ok(self.populate_highlights(plants))
    }

    pub async fn find_plant(&self, id: usize) -> Option<Plant> {
        info!("find_plant {id}");

        let plant = self.db.get_plant_by_id(id).await?;
        Some(Plant {
            highlights: self.highlights.generate(&plant),
            ..plant
        })
    }

    async fn get_closest_valid_zip(&self, zip: &str) -> anyhow::Result<String> {
        let valid_zip = self.db.lookup_closest_valid_zip(zip).await.map_err(|e| {
            warn!("Cannot find valid zipcode: {e}");
            anyhow::anyhow!("invalid zipcode")
        })?;

        if valid_zip != zip {
            info!("Adjusted unknown zip {} to {valid_zip}", zip);
        }

        Ok(valid_zip)
    }

    fn populate_highlights(&self, plants: Vec<Plant>) -> Vec<Plant> {
        plants
            .into_iter()
            .map(|p| Plant {
                highlights: self.highlights.generate(&p),
                ..p
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct PlantSearchCommand {
    pub name: Option<String>,
    pub zip: Option<String>,
    pub shade: Option<Shade>,
    pub moisture: Option<Moisture>,
    pub habit: Option<Habit>,
}
