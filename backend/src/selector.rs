use crate::ai;
use crate::database::Database;
use crate::domain::*;
use futures::stream::{self, Stream, StreamExt};
use futures::{future, Future};
use std::boxed::Box;
use std::collections::HashSet;
use std::env;
use std::pin::Pin;
use tracing::log::{info, warn};

pub struct PlantStream {
    pub stream: Pin<Box<dyn Stream<Item = Plant> + Send>>,
    pub from_db: bool,
}

pub async fn stream_plants(
    db: &Database,
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<PlantStream> {
    // Fetch a stream of plants from some combination of the db and llm.
    // These items may or may not be filtered by shade/moisture.  They won't be
    // filtered if they came from the LLM or if the db lacked condition info.
    let unfiltered_stream = get_unfiltered_plant_stream(db, zip, moisture, shade).await?;

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
        .then(update_plant_from_database(db.clone()))
        .map(update_plant_with_conditions)
        .buffer_unordered(12)
        .then(save_plant(db.clone()))
        .filter(move |plant| {
            //TODO: If conditions are still unknown, should I give it the benefit
            //      of the doubt?  Currently no.
            let should_keep = plant.moistures.contains(&moisture) && plant.shades.contains(&shade);

            if !should_keep {
                info!(
                    "Filtered out {} ({}) because it does not thrive in moisture: {} and shade: {}",
                    plant.common, plant.scientific, moisture, shade
                );
            }
            future::ready(should_keep)
        });

    Ok(PlantStream {
        stream: Box::pin(filtered_stream),
        from_db: unfiltered_stream.from_db,
    })
}

fn update_plant_from_database(
    db: Database,
) -> impl FnMut(Plant) -> Pin<Box<dyn Future<Output = Plant> + Send>> {
    move |plant: Plant| {
        let db_clone = db.clone();
        Box::pin(async move {
            if plant.id.is_some() {
                return plant; // This plant came from the database, don't fetch it again.
            }

            let fetch_future = db_clone.get_plant_by_scientific_name(&plant.scientific);
            if let Some(existing_plant) = fetch_future.await {
                existing_plant // Plant found in db
            } else {
                plant // Plant not found in db, return what we had
            }
        })
    }
}

fn save_plant(db: Database) -> impl FnMut(Plant) -> Pin<Box<dyn Future<Output = Plant> + Send>> {
    move |plant: Plant| {
        let db_clone = db.clone();
        Box::pin(async move {
            match db_clone.save_plant(&plant).await {
                Ok(updated_plant) => updated_plant,
                Err(e) => {
                    warn!("Failed to save plant: {e}");
                    plant
                }
            }
        })
    }
}

async fn get_unfiltered_plant_stream(
    db: &Database,
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<PlantStream> {
    let db_plants = db.lookup_query_results(zip, moisture, shade).await;
    //db_plants.sort_by_key(|p| Reverse(p.score()));

    // If this query has been executed "enough", then return the database
    // results without consulting the llm.
    if db.get_query_count(zip, moisture, shade).await >= 3 {
        //db_plants.sort_by_key(|p| Reverse(p.score()));

        return Ok(PlantStream {
            stream: Box::pin(stream::iter(db_plants)),
            from_db: true,
        });
    }

    let db = db.clone();
    let llm_stream = get_llm_plant_stream(&db, zip, moisture, shade).await?;

    // Chain the db & llm streams together.  All the db results be available
    // quickly, then the llm results will trickle in.
    let merged_stream = stream::iter(db_plants).chain(llm_stream);

    Ok(PlantStream {
        stream: Box::pin(merged_stream),
        from_db: false, // Not everything is from the database, hence false
    })
}

async fn update_plant_with_conditions(plant: Plant) -> Plant {
    if !plant.shades.is_empty() && !plant.moistures.is_empty() {
        // If they're already populated, nothing needs to be done.
        return plant;
    }

    let openai_api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");

    match ai::fetch_conditions(&openai_api_key, &plant.scientific).await {
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

async fn get_llm_plant_stream(
    db: &Database,
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<impl Stream<Item = Plant>> {
    let openai_api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");

    let region_name = db
        .get_region_name_by_zip(zip)
        .await
        .unwrap_or_else(|| format!("US Zip Code {zip}"));

    let plants = ai::stream_plants(
        &openai_api_key,
        &region_name,
        shade.description(),
        moisture.description(),
    )
    .await?;

    Ok(plants)
}
