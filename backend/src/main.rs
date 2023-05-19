use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use futures::executor::block_on;
use openai::NativePlantEntry;
use serde::{Deserialize, Serialize};
use std::env;
use std::thread;
use std::time;

mod flickr;
mod openai;

#[derive(Serialize, Deserialize, Debug)]
struct FetchRequest {
    zip: String,
    shade: String,
    moisture: String,
}

#[get("/plants")]
async fn fetch_entries_handler(web::Query(payload): web::Query<FetchRequest>) -> impl Responder {
    println!("Received request: {:#?}", payload);

    let (sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(10);

    thread::spawn(move || {
        let openai_api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
        let flickr_api_key = env::var("FLICKR_API_KEY").expect("Must define $OPENAI_API_KEY");

        let entries = openai::stream_entries(
            &openai_api_key,
            &payload.zip,
            &payload.shade,
            &payload.moisture,
        );

        for entry in entries {
            let entry_json = serde_json::to_string(&entry).unwrap();
            //TODO: Can I get rid of the "block on"?
            //      I think I need this thread to be async?

            block_on(sender.send(sse::Data::new(entry_json))).unwrap();

            if let Some(image_url) = flickr::get_image_url(&entry.scientific, &flickr_api_key) {
                block_on(
                    sender.send(
                        sse::Data::new(format!("{}::{}", entry.scientific, image_url))
                            .event("image_url"),
                    ),
                )
                .unwrap();
            }
        }

        block_on(sender.send(sse::Data::new("").event("close"))).unwrap();
    });

    stream.with_keep_alive(time::Duration::from_secs(1))
}

#[get("/plants_mock")]
async fn fetch_entries_handler_mock(
    web::Query(payload): web::Query<FetchRequest>,
) -> impl Responder {
    println!("Received mock request: {:#?}", payload);

    let (sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(2);

    thread::spawn(move || {
        for mut entry in build_mock_plants() {
            // Remove image to let it load a moment later.
            let image_url = entry.image_url;
            entry.image_url = None;

            let entry_json = serde_json::to_string(&entry).unwrap();

            block_on(sender.send(sse::Data::new(entry_json))).unwrap();
            thread::sleep(time::Duration::from_millis(250));

            block_on(
                sender.send(
                    sse::Data::new(format!("{}::{}", entry.scientific, image_url.unwrap()))
                        .event("image_url"),
                ),
            )
            .unwrap();
            thread::sleep(time::Duration::from_millis(500));
        }

        block_on(sender.send(sse::Data::new("").event("close"))).unwrap();
    });

    stream.with_keep_alive(time::Duration::from_secs(1))
        .customize()
        .insert_header(("X-Accel-Buffering", "no"))
}

fn build_mock_plants() -> impl Iterator<Item = NativePlantEntry> {
    vec![
        NativePlantEntry {
            common: "Wild Columbine".to_string(),
            scientific: "Aquilegia canadensis".to_string(),
            bloom: "Spring to early summer".to_string(),
            description: "This plant is a favorite of hummingbirds and supports the Columbine Duskywing butterfly caterpillar.".to_string(),
            image_url: Some("https://live.staticflickr.com/5031/7238526710_80bf103077_q.jpg".to_string()),
        },
        NativePlantEntry {
            common: "Swamp Milkweed".to_string(),
            scientific: "Asclepias incarnata".to_string(),
            bloom: "Summer".to_string(),
            description: "This plant is a host for the Monarch butterfly caterpillar and supports many other pollinators.".to_string(),
            image_url: Some("https://live.staticflickr.com/3126/3147197425_4e9ac1e2ca_q.jpg".to_string()),
        },
        NativePlantEntry {
            common: "Joe Pye Weed".to_string(),
            scientific: "Eutrochium purpureum".to_string(),
            bloom: "Late summer to fall".to_string(),
            description: "This plant is a favorite of many pollinators, including bees and butterflies.".to_string(),
            image_url: Some("https://live.staticflickr.com/3862/15215414361_9f659f6f52_q.jpg".to_string()),
        },
        NativePlantEntry {
            common: "Blue Flag Iris".to_string(),
            scientific: "Iris versicolor".to_string(),
            bloom: "Late spring to early summer".to_string(),
            description: "This plant supports the Baltimore Checkerspot butterfly caterpillar and is a favorite of many pollinators.".to_string(),
            image_url: Some("https://live.staticflickr.com/65535/50623901946_1c37f69ccd_q.jpg".to_string()),
        },
        NativePlantEntry {
            common: "Cardinal Flower".to_string(),
            scientific: "Lobelia cardinalis".to_string(),
            bloom: "Late summer to early fall".to_string(),
            description: "This plant is a favorite of hummingbirds and supports many other pollinators.".to_string(),
            image_url: Some("https://live.staticflickr.com/6174/6167236354_c7e9771f00_q.jpg".to_string()),
        }
    ].into_iter()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            //TODO: Don't do this in prod... but it lets me skip using the
            //      React proxy server which causes issues with streaming events
            //.wrap(Cors::default().allowed_origin("http://localhost:3000"))
            .wrap(Cors::permissive())
            .service(fetch_entries_handler)
            .service(fetch_entries_handler_mock)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
