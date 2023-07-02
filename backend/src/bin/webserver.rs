use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use futures::join;
use futures::stream::{Stream, StreamExt};
use openai::NativePlant;
use planting_life::citations;
use planting_life::flickr;
use planting_life::openai;
use serde::{self, Deserialize, Serialize};
use std::boxed::Box;
use std::env;
use std::pin::Pin;
use std::time;
use tracing::{info, warn};

#[derive(Serialize, Deserialize, Debug)]
struct FetchRequest {
    zip: String,
    shade: Shade,
    moisture: Moisture,
}

#[derive(Serialize, Deserialize, Debug)]
enum Shade {
    #[serde(rename = "Full Shade")]
    Full,
    #[serde(rename = "Partial Shade")]
    Partial,
    #[serde(rename = "Full Sun")]
    No,
}

impl Shade {
    fn description(&self) -> &str {
        match self {
            Shade::Full => "full shade",
            Shade::Partial => "partial shade",
            Shade::No => "full sun",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Moisture {
    Low,
    Medium,
    High,
}

impl Moisture {
    fn description(&self) -> &str {
        match self {
            Moisture::Low => "dry soil",
            Moisture::Medium => "moderately wet soil",
            Moisture::High => "wet soil",
        }
    }
}

#[get("/plants")]
async fn fetch_plants_handler(web::Query(payload): web::Query<FetchRequest>) -> impl Responder {
    info!("{payload:?}");

    //TODO: 10 might be small now that I'm streaming descriptions back.
    let (sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(10);

    // The real work is done in a new thread so the connection to the front end can open.
    actix_web::rt::spawn(async move {
        let mut plants = match get_plant_stream(payload).await {
            Ok(plants) => plants,
            Err(err) => {
                warn!("Failed to get plant stream {err}");
                send_event(&sender, "error", "").await;
                return;
            }
        };

        let mut handles = vec![];
        while let Some(plant) = plants.next().await {
            // Make a clone, so the inner and outer tasks can each own a sender
            let sender_clone = sender.clone();

            // This inner task is started so the next entry can start processing before
            // the previous one finishes.
            let handle = actix_web::rt::spawn(async move {
                // Concurrently send the plant to the front end while handling the image
                join!(
                    send_plant(&sender_clone, &plant),
                    fetch_and_send_image(&sender_clone, &plant),
                    fetch_and_send_description(&sender_clone, &plant),
                    fetch_and_send_citations(&sender_clone, &plant),
                );
            });

            handles.push(handle);
        }

        send_event(&sender, "allPlantsLoaded", "").await;

        // Wait for all inner tasks to finish before closing the stream
        // This lets any outstanding data be written back to the client
        for handle in handles {
            handle.await.unwrap_or_default();
        }

        send_event(&sender, "close", "").await;
    });

    stream
        .with_keep_alive(time::Duration::from_secs(1))
        .customize()
        .insert_header(("X-Accel-Buffering", "no"))
}

async fn get_plant_stream(
    payload: FetchRequest,
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

async fn fetch_and_send_image(sender: &Sender, plant: &NativePlant) {
    let flickr_api_key = env::var("FLICKR_API_KEY").expect("Must define $FLICKR_API_KEY");

    if let Some(image) = flickr::get_image(&plant.scientific, &plant.common, &flickr_api_key).await
    {
        let image_json = serde_json::to_string(&image).expect("image should serialize");
        send_event(sender, "image", &image_json).await;
    }
}

async fn fetch_and_send_description(sender: &Sender, plant: &NativePlant) {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let description_stream = match openai::fetch_description(&api_key, &plant.scientific).await {
        Ok(stream) => stream,
        Err(_) => {
            warn!("Failed to fetch description");
            return;
        }
    };

    let mut description_stream = Box::pin(description_stream);
    while let Some(description_delta) = description_stream.next().await {
        let payload = format!(
            r#"{{"scientificName": "{}", "descriptionDelta": "{}"}}"#,
            plant.scientific, description_delta
        );

        send_event(sender, "descriptionDelta", &payload).await;
    }
}

async fn fetch_and_send_citations(sender: &Sender, plant: &NativePlant) {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("https://www.planting.life")
            .allowed_origin("https://planting.life")
            .allowed_methods(vec!["GET"]);

        App::new().wrap(cors).service(fetch_plants_handler)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
