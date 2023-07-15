use crate::database::Database;
use crate::domain::*;
use crate::openai;
use futures::stream::{self, Stream};
use std::boxed::Box;
use std::env;
use std::pin::Pin;

pub struct PlantStream {
    pub stream: Pin<Box<dyn Stream<Item = NativePlant> + Send>>,
    pub from_db: bool,
}

pub async fn stream_plants(
    db: &Database,
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<PlantStream> {
    let plants_from_db = db.lookup_query_results(zip, moisture, shade).await;
    if !plants_from_db.is_empty() {
        return Ok(PlantStream {
            stream: Box::pin(stream::iter(plants_from_db)),
            from_db: true,
        });
    }

    Ok(PlantStream {
        stream: Box::pin(get_llm_plant_stream(db, zip, moisture, shade).await?),
        from_db: false,
    })
}

async fn get_llm_plant_stream(
    db: &Database,
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<impl Stream<Item = NativePlant>> {
    let openai_api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");

    let region_name = db
        .get_region_name_by_zip(zip)
        .await
        .unwrap_or_else(|| format!("US Zip Code {zip}"));

    let plants = openai::stream_plants(
        &openai_api_key,
        &region_name,
        shade.description(),
        moisture.description(),
    )
    .await?;

    Ok(plants)
}
