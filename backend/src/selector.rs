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
        stream: Box::pin(get_llm_plant_stream(zip, moisture, shade).await?),
        from_db: false,
    })
}

async fn get_llm_plant_stream(
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<impl Stream<Item = NativePlant>> {
    let openai_api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");

    //TODO: Lookup region name, pass that to stream_plants instead of zip.

    let plants = openai::stream_plants(
        &openai_api_key,
        zip,
        shade.description(),
        moisture.description(),
    )
    .await?;

    Ok(plants)
}
