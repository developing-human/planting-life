use crate::ai::Ai;
use crate::database::Database;
use crate::domain::*;
use async_trait::async_trait;
use futures::stream::{self, Stream, StreamExt};
use futures::{future, Future};
use std::boxed::Box;
use std::collections::HashSet;
use std::pin::Pin;
use tracing::log::warn;

pub struct PlantStream {
    pub stream: Pin<Box<dyn Stream<Item = Plant> + Send>>,
    pub from_db: bool,
}

#[async_trait]
pub trait Selector: Send + Sync {
    async fn stream_plants(
        &'static self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> anyhow::Result<PlantStream>;
}

pub struct RealSelector {
    db: &'static dyn Database,
    ai: &'static dyn Ai,
}

impl RealSelector {
    pub fn new(db: &'static dyn Database, ai: &'static dyn Ai) -> Self {
        Self { db, ai }
    }
}

#[async_trait]
impl Selector for RealSelector {
    async fn stream_plants(
        &'static self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> anyhow::Result<PlantStream> {
        // Fetch a stream of plants from some combination of the db and llm.
        // These items may or may not be filtered by shade/moisture.  They won't be
        // filtered if they came from the LLM or if the db lacked condition info.
        let unfiltered_stream = self
            .get_unfiltered_plant_stream(zip, moisture, shade)
            .await?;

        let moisture = moisture.to_owned();
        let shade = shade.to_owned();

        let mut seen_common_names = HashSet::new();

        let filtered_stream = unfiltered_stream
            .stream
            .filter(move |plant| {
                // Filter out common names that have already been seen.  Often, we'll seen
                // something like Joe Pye Weed come through via 2-3 scientific names
                let mut sanitized = plant.common.to_lowercase();
                sanitized.retain(|c| c.is_alphabetic());

                let never_seen_before = seen_common_names.insert(sanitized);
                future::ready(never_seen_before)
            })
            .then(self.update_plant_from_database())
            .map(|p| self.update_plant_with_conditions(p))
            .buffer_unordered(12)
            .then(self.save_plant(zip.to_string()))
            .filter(move |plant| {
                let should_keep =
                    plant.moistures.contains(&moisture) && plant.shades.contains(&shade);
                future::ready(should_keep)
            });

        Ok(PlantStream {
            stream: Box::pin(filtered_stream),
            from_db: unfiltered_stream.from_db,
        })
    }
}

impl RealSelector {
    async fn get_llm_plant_stream(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Plant> + Send>>> {
        let region_name = self
            .db
            .get_region_name_by_zip(zip)
            .await
            .unwrap_or_else(|| format!("US Zip Code {zip}"));

        let plants = self
            .ai
            .stream_plants(&region_name, shade.description(), moisture.description())
            .await?;

        Ok(plants)
    }

    async fn get_unfiltered_plant_stream(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> anyhow::Result<PlantStream> {
        let db_plants = self.db.lookup_query_results(zip, moisture, shade).await;

        // If this query has been executed "enough", then return the database
        // results without consulting the llm.
        if self.db.get_query_count(zip, moisture, shade).await >= 3 {
            return Ok(PlantStream {
                stream: Box::pin(stream::iter(db_plants)),
                from_db: true,
            });
        }

        // I have never seen anything return from the LLM for full shade and
        // low moisture.  Not in the midwest, not in death valley.  Don't even
        // try.  But if something was found in the database from another search
        // go ahead and return it.
        if *moisture == Moisture::None && *shade == Shade::Lots {
            return Ok(PlantStream {
                stream: Box::pin(stream::iter(db_plants)),
                from_db: true,
            });
        }

        let llm_stream = self.get_llm_plant_stream(zip, moisture, shade).await?;

        // Chain the db & llm streams together.  All the db results be available
        // quickly, then the llm results will trickle in.
        let merged_stream = stream::iter(db_plants).chain(llm_stream);

        Ok(PlantStream {
            stream: Box::pin(merged_stream),
            from_db: false, // Not everything is from the database, hence false
        })
    }

    fn update_plant_from_database(
        &'static self,
    ) -> impl FnMut(Plant) -> Pin<Box<dyn Future<Output = Plant> + Send>> {
        move |plant: Plant| {
            Box::pin(async move {
                if plant.id.is_some() {
                    return plant; // This plant came from the database, don't fetch it again.
                }

                let fetch_future = self.db.get_plant_by_scientific_name(&plant.scientific);
                if let Some(existing_plant) = fetch_future.await {
                    existing_plant // Plant found in db
                } else {
                    plant // Plant not found in db, return what we had
                }
            })
        }
    }

    fn save_plant(
        &'static self,
        zip: String,
    ) -> impl FnMut(Plant) -> Pin<Box<dyn Future<Output = Plant> + Send>> {
        move |plant: Plant| {
            let zip = zip.clone();

            Box::pin(async move {
                let plant = match self.db.save_plant(&plant).await {
                    Ok(updated_plant) => updated_plant,
                    Err(e) => {
                        warn!("Failed to save plant: {e}");
                        plant
                    }
                };

                self.db.save_plant_region(&plant, &zip).await;

                plant
            })
        }
    }

    async fn update_plant_with_conditions(&self, plant: Plant) -> Plant {
        if !plant.shades.is_empty() && !plant.moistures.is_empty() {
            // If they're already populated, nothing needs to be done.
            return plant;
        }

        match self.ai.fetch_conditions(&plant.scientific).await {
            Ok(conditions) => Plant {
                moistures: conditions.moisture,
                shades: conditions.shade,
                ..plant
            },
            Err(e) => {
                warn!("Could not get conditions for plant: {e}");
                plant
            }
        }
    }
}
