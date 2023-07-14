use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use futures::channel::mpsc;
use futures::stream::Stream;
use futures::StreamExt;
use planting_life::database::Database;
use planting_life::domain::{Moisture, NativePlant, Shade};
use planting_life::hydrator;
use planting_life::selector;
use serde::{self, Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::time;
use tracing::{info, warn};

#[derive(Serialize, Deserialize, Debug)]
struct PlantsRequest {
    zip: String,
    shade: Shade,
    moisture: Moisture,
}

#[get("/plants")]
async fn fetch_plants_handler(
    web::Query(payload): web::Query<PlantsRequest>,
    db: web::Data<Database>,
) -> impl Responder {
    info!("{payload:?}");

    let (frontend_sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(10);
    let db = db.get_ref().clone();

    // The real work is done in a new thread so the connection to the front end can stay open.
    actix_web::rt::spawn(async move {
        let plant_stream =
            selector::stream_plants(&db, &payload.zip, &payload.moisture, &payload.shade).await;

        match plant_stream {
            Ok(plant_stream) => {
                fill_and_send_plants(
                    &db,
                    &payload,
                    plant_stream.stream,
                    &frontend_sender,
                    plant_stream.from_db,
                )
                .await
            }
            Err(_) => send_event(&frontend_sender, "error", "").await,
        };

        send_event(&frontend_sender, "close", "").await;
    });

    stream
        .with_keep_alive(time::Duration::from_secs(1))
        .customize()
        .insert_header(("X-Accel-Buffering", "no"))
}

async fn fill_and_send_plants(
    db: &Database,
    payload: &PlantsRequest,
    plant_stream: impl Stream<Item = NativePlant> + 'static + Unpin,
    frontend_sender: &Sender,
    plants_from_db: bool,
) {
    let (mut plant_sender, mut plant_receiver) = mpsc::unbounded();
    let db_clone = db.clone();

    actix_web::rt::spawn(async move {
        hydrator::hydrate_plants(&db_clone, plant_stream, &mut plant_sender).await;
    });
    let mut plants_to_save = vec![];
    let mut all_plants = vec![];
    while let Some(hydrated_plant) = plant_receiver.next().await {
        if hydrated_plant.done {
            all_plants.push(hydrated_plant.plant.clone());

            if hydrated_plant.updated {
                plants_to_save.push(hydrated_plant.plant.clone());
            }
        }

        send_plant(frontend_sender, &hydrated_plant.plant).await;
    }

    // Save any plants which are new or updated.  If any fail, don't cache the query results.
    // This is because missing ids will result in a partial cache.

    let saved_plants = match db.save_plants(&plants_to_save.iter().collect()).await {
        Ok(saved_plants) => saved_plants,
        Err(e) => {
            warn!("failed to save plants, not caching: {e}");
            return;
        }
    };

    // We only need to cache the query results if these results aren't from the database
    // When they are from the database, we know its already there.
    if plants_from_db {
        return; // not logging, this is very common
    }

    // Also, don't save the results of this query if we have fewer than the desired number,
    // this should be a rare occurance and this is a simple way to handle it.  The
    // alternative would be to (on fetch) get some from the database and the rest from gpt.
    // Its easier to just get all from gpt, even if its a little more work.
    if all_plants.len() != 12 {
        info!("only have {} plants, not caching", all_plants.len());
        return;
    }

    // At least one plant is missing an image, so don't store these results.  Very
    // occasionally we'll run into this, and its okay as a quirk but lets not cache
    // it forever.
    let plant_without_image = all_plants.iter().find(|p| p.image.is_none());
    if let Some(plant_without_image) = plant_without_image {
        info!(
            "not all plants have an image (missing for {}), not caching",
            plant_without_image.scientific
        );
        return;
    }

    let plant_ids: HashSet<usize> = all_plants
        .iter()
        .chain(saved_plants.iter())
        .filter_map(|p| p.id)
        .collect();

    db.save_query_results(&payload.zip, &payload.moisture, &payload.shade, plant_ids)
        .await;
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

    let db_url = match env::var("PLANTING_LIFE_DB_URL") {
        Ok(s) => s,
        _ => {
            warn!("Configure valid PLANTING_LIFE_DB_URL to use database");
            "".to_string()
        }
    };
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
