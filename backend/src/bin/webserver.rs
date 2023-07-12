use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use async_mutex::Mutex;
use futures::join;
use futures::stream::{self, Stream, StreamExt};
use planting_life::database::Database;
use planting_life::domain::{Image, Moisture, NativePlant, Shade};
use planting_life::flickr;
use planting_life::openai;
use serde::{self, Deserialize, Serialize};
use std::boxed::Box;
use std::collections::HashSet;
use std::env;
use std::pin::Pin;
use std::sync::Arc;
use std::time;
use tracing::{info, warn};

#[derive(Serialize, Deserialize, Debug)]
struct PlantsRequest {
    zip: String,
    shade: Shade,
    moisture: Moisture,
}

async fn fill_and_send_plants(
    db: web::Data<Database>,
    payload: &PlantsRequest,
    mut plants: impl Stream<Item = NativePlant> + Unpin,
    sender: &Sender,
    plants_from_db: bool,
) {
    let db = web::Data::new(Arc::new(db));

    // Holds (and owns) all the plants which are returned.
    let all_plants = Arc::new(Mutex::new(vec![]));

    // Holds references to plants which are either new or updated
    let plants_to_save = Arc::new(Mutex::new(vec![]));

    // References to tasks which are running
    let mut handles = vec![];

    while let Some(plant) = plants.next().await {
        // Make a clone, so the inner and outer tasks can each own a sender
        let sender_clone = sender.clone();
        let db = db.get_ref().clone();

        let all_plants = all_plants.clone();
        let plants_to_save = plants_to_save.clone();

        // This inner task is started so the next entry can start processing before
        // the previous one finishes.
        let handle = actix_web::rt::spawn(async move {
            // If this plant didn't come from the datbase, check the database for it.
            let mut plant = plant;
            if plant.id.is_none() {
                let existing_plant = db.get_plant_by_scientific_name(&plant.scientific).await;
                if let Some(existing_plant) = existing_plant {
                    plant = existing_plant;
                }
            }

            // At this point I have a plant from the gpt list + database query
            // Some parts could be missing (not in db, db is missing parts)
            // Now, any missing parts need to be filled in.

            // Concurrently send the plant to the front end while handling the image
            let (_, img, description /*, _*/) = join!(
                send_plant(&sender_clone, &plant),
                fetch_and_send_image(&sender_clone, &plant),
                fetch_and_send_description(&sender_clone, &plant),
                //TODO: Bring citations back once they can be cached or displayed.
                //fetch_and_send_citations(&sender_clone, &plant),
            );

            let updated_plant = NativePlant {
                image: img,
                description,
                ..plant
            };

            // Only save plants which weren't from the database (no id) or where
            // the description was updated.  In the future, we'll also want to check
            // images and citations once those are handled by the db.
            if updated_plant.id.is_none()
                || updated_plant.description != plant.description
                || (plant.image.is_none() && updated_plant.image.is_some())
            {
                plants_to_save.lock().await.push(updated_plant.clone());
            }

            all_plants.lock().await.push(updated_plant);
        });

        handles.push(handle);
    }

    send_event(sender, "allPlantsLoaded", "").await;

    // Wait for all inner tasks to finish before closing the stream
    // This lets any outstanding data be written back to the client
    for handle in handles {
        handle.await.unwrap_or_default();
    }

    let plants_to_save = plants_to_save.lock().await;
    let all_plants = all_plants.lock().await;

    // Save any plants which are new or updated.
    let saved_plants = db.save_plants(&plants_to_save.iter().collect()).await;

    // We only need to cache the query results if these results aren't from the database
    // When they are from the database, we know its already there.
    if !plants_from_db {
        let plant_ids: HashSet<usize> = all_plants
            .iter()
            .chain(saved_plants.iter())
            .filter_map(|p| p.id)
            .collect();

        db.save_query_results(&payload.zip, &payload.moisture, &payload.shade, plant_ids)
            .await;
    }
}

#[get("/plants")]
async fn fetch_plants_handler(
    web::Query(payload): web::Query<PlantsRequest>,
    db: web::Data<Database>,
) -> impl Responder {
    info!("{payload:?}");

    //TODO: 10 might be small now that I'm streaming descriptions back.
    let (sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(1);

    // The real work is done in a new thread so the connection to the front end can open.
    actix_web::rt::spawn(async move {
        let plants_db = db
            .lookup_query_results(&payload.zip, &payload.moisture, &payload.shade)
            .await;

        if !plants_db.is_empty() {
            fill_and_send_plants(db, &payload, stream::iter(plants_db), &sender, true).await;
        } else {
            let plants = match get_plant_stream(&payload).await {
                Ok(plants) => plants,
                Err(err) => {
                    warn!("Failed to get plant stream {err}");
                    send_event(&sender, "error", "").await;
                    return;
                }
            };

            fill_and_send_plants(db, &payload, plants, &sender, false).await;
        }

        send_event(&sender, "close", "").await;
    });

    stream
        .with_keep_alive(time::Duration::from_secs(1))
        .customize()
        .insert_header(("X-Accel-Buffering", "no"))
}

async fn get_plant_stream(
    payload: &PlantsRequest,
) -> anyhow::Result<Pin<Box<impl Stream<Item = NativePlant>>>> {
    let openai_api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");

    let plants = openai::stream_plants(
        &openai_api_key,
        &payload.zip,
        payload.shade.description(),
        payload.moisture.description(),
    )
    .await?;

    // I don't yet understand why this is needed.  But it tells the plants
    // not to move in memory during async work.
    Ok(Box::pin(plants))
}

async fn send_plant(sender: &Sender, plant: &NativePlant) {
    let json = serde_json::to_string(&plant).expect("plant should serialize");

    send_event(sender, "plant", &json).await;
}

async fn send_event(sender: &Sender, event: &str, message: &str) {
    let data = sse::Data::new(message).event(event);

    match sender.send(data).await {
        Ok(_) => {}
        Err(_) => warn!("Error sending [{}] with message [{}]", event, message),
    }
}

async fn fetch_and_send_image(sender: &Sender, plant: &NativePlant) -> Option<Image> {
    if plant.image.is_some() {
        info!("Skipping image fetch for: {}", plant.scientific);
        // Don't fetch or send if its already available
        // If already populated, its been sent w/ the original plant
        return plant.image.clone();
    }
    info!("Fetching image for: {}", plant.scientific);

    let flickr_api_key = env::var("FLICKR_API_KEY").expect("Must define $FLICKR_API_KEY");

    let image = flickr::get_image(&plant.scientific, &plant.common, &flickr_api_key).await;
    if let Some(image) = image {
        let image_json = serde_json::to_string(&image).expect("image should serialize");
        send_event(sender, "image", &image_json).await;

        return Some(image);
    }

    None
}

async fn fetch_and_send_description(sender: &Sender, plant: &NativePlant) -> Option<String> {
    if plant.description.is_some() {
        // Don't fetch or send if its already available
        // If already populated, its been sent w/ the original plant
        return plant.description.clone();
    }

    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let description_stream = match openai::fetch_description(&api_key, &plant.scientific).await {
        Ok(stream) => stream,
        Err(_) => {
            warn!("Failed to fetch description");
            return None;
        }
    };

    let mut description_deltas = vec![];
    let mut description_stream = Box::pin(description_stream);
    while let Some(description_delta) = description_stream.next().await {
        let payload = format!(
            r#"{{"scientificName": "{}", "descriptionDelta": "{}"}}"#,
            plant.scientific, description_delta
        );

        description_deltas.push(description_delta);
        send_event(sender, "descriptionDelta", &payload).await;
    }

    if description_deltas.is_empty() {
        None
    } else {
        Some(description_deltas.join(""))
    }
}

/*
async fn fetch_and_send_citations(sender: &Sender, plant: &NativePlant) {
    //TODO: I think citations::find needs to know what citations we already have,
    //      and only try to build out the ones we don't have.  But currently we
    //      don't even have citations in the db.
    let citations = citations::find(&plant.scientific).await;
    if !citations.is_empty() {
        let citations_json = serde_json::to_string(&citations).expect("citations should serialize");
        let payload = format!(
            r#"{{"scientificName": "{}", "citations": {}}}"#,
            plant.scientific, citations_json
        );
        send_event(sender, "citations", &payload).await;
    }
}
*/

#[derive(Serialize, Deserialize, Debug)]
struct NurseriesRequest {
    zip: String,
}

#[get("/nurseries")]
async fn fetch_nurseries_handler(
    web::Query(payload): web::Query<NurseriesRequest>,
    db: web::Data<Database>,
) -> impl Responder {
    info!("{payload:?}");

    let mut nurseries = db.find_nurseries(&payload.zip).await;

    // Some areas have 20+ nurseries and it looks ridiculous, set a limit
    nurseries.truncate(10);

    for nursery in &mut nurseries {
        if nursery.map_url.is_none() {
            // Pad the zip code to five digits, using zeros.
            let zip = format!("{:05}", nursery.zip);

            let query = format!("{} near {}", nursery.name, zip);
            let query = query.replace(' ', "+");
            let url = format!("https://www.google.com/maps/search/?api=1&query={query}");

            nursery.map_url = Some(url);
        }
    }

    actix_web::HttpResponse::Ok().json(nurseries)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let db_url = env::var("PLANTING_LIFE_DB_URL").expect("Must define $PLANTING_LIFE_DB_URL");
    let db = Database::new(&db_url);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://www.planting.life")
            .allowed_origin("https://planting.life")
            .allowed_methods(vec!["GET"]);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(db.clone()))
            .service(fetch_plants_handler)
            .service(fetch_nurseries_handler)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
