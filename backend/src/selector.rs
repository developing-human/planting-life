use crate::database::Database;
use crate::domain::*;
use crate::openai;
use futures::stream::{self, Stream};
use std::boxed::Box;
use std::env;
use std::pin::Pin;

pub async fn stream_plants(
    db: &Database,
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<Pin<Box<dyn Stream<Item = NativePlant> + Send>>> {
    // The complexity of this rather simple operation is due to the database
    // and llm returning different types of streams
    let plants_from_db = db.lookup_query_results(zip, moisture, shade).await;

    if !plants_from_db.is_empty() {
        println!("got stream from db");
        return Ok(Box::pin(stream::iter(plants_from_db)));
    }

    println!("getting llm stream");
    Ok(Box::pin(get_llm_plant_stream(zip, moisture, shade).await?))
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

    println!("finishing get_llm_plant_stream");

    // I don't yet understand why this is needed.  But it tells the plants
    // not to move in memory during async work.
    Ok(plants)
}
