use crate::ai;
use crate::database::Database;
use crate::domain::*;
use futures::stream::{self, Stream, StreamExt};
use std::boxed::Box;
use std::cmp::Reverse;
use std::env;
use std::pin::Pin;

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
    let mut plants_from_db = db.lookup_query_results(zip, moisture, shade).await;
    if !plants_from_db.is_empty() {
        plants_from_db.sort_by_key(|p| Reverse(p.score()));

        return Ok(PlantStream {
            stream: Box::pin(stream::iter(plants_from_db)),
            from_db: true,
        });
    }

    let db = db.clone();
    let plant_stream = get_llm_plant_stream(&db, zip, moisture, shade)
        .await?
        .then(move |plant| {
            // This splits the "move" from "async move" so the db reference
            // can be cloned and shared between threads.
            let db_clone = db.clone();
            async move {
                let fetch_future = db_clone.get_plant_by_scientific_name(&plant.scientific);
                if let Some(existing_plant) = fetch_future.await {
                    existing_plant
                } else {
                    plant
                }
            }
        });

    Ok(PlantStream {
        stream: Box::pin(plant_stream),
        from_db: false,
    })
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
