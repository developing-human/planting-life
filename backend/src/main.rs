use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use futures::executor::block_on;
use openai::NativePlantEntry;
use serde::{self, Deserialize, Serialize};
use std::env;
use std::thread;
use std::time;

mod flickr;
mod openai;

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
            Moisture::Medium => "moist soil",
            Moisture::High => "wet soil",
        }
    }
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
            payload.shade.description(),
            payload.moisture.description(),
        );

        for entry in entries {
            let entry_json = serde_json::to_string(&entry).unwrap();

            //TODO: Can I get rid of the "block on"?
            //      I think I need this thread to be async?

            block_on(sender.send(sse::Data::new(entry_json))).unwrap();

            if let Some(image) =
                flickr::get_image(&entry.scientific, &entry.common, &flickr_api_key)
            {
                // TODO: Remove image_url event once front end supports image event
                block_on(
                    sender.send(
                        sse::Data::new(format!("{}::{}", entry.scientific, image.thumbnail_url))
                            .event("image_url"),
                    ),
                )
                .unwrap();

                let image_json = serde_json::to_string(&image).unwrap();
                block_on(sender.send(sse::Data::new(image_json).event("image"))).unwrap();
            }
        }

        block_on(sender.send(sse::Data::new("").event("close"))).unwrap();
    });

    stream
        .with_keep_alive(time::Duration::from_secs(1))
        .customize()
        .insert_header(("X-Accel-Buffering", "no"))
}

#[get("/plants_mock")]
async fn fetch_entries_handler_mock(
    web::Query(payload): web::Query<FetchRequest>,
) -> impl Responder {
    println!("Received mock request: {:#?}", payload);

    let (sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(2);

    // Pointing all mock attributions to the same image while I wire up something reasonable
    let image = flickr::Image {
        scientific_name: String::from("Asclepias incarnata"),
        title: String::from("Milkweed"),
        thumbnail_url: String::from("https://live.staticflickr.com/71/175851524_04904b8383_q_d.jpg"),
        original_url: String::from("https://www.flickr.com/photos/salim/175851524/in/photolist-LYSddJ-do77yy-gxh1X-gxhhQ-gxhwQ-29aXjBh-gxiXt-gxjzi-2gTsfpW-2nu6TdF-t7vKB-gQpY2t-2jxkYNQ-Vxf8qd-yn9kQ6-25wQste-2jxgKNA-rpQqA4-BsgvHn-pPbpVq-dwwp6P-dwBTEW-66cBaE-7iwAYm-7dcY49-6AaZnj-6AaZQw-5amQLW-6A6R9F-4PhsgS-2m57JQa-tZkSa-dwBUjf-dwBW3f-69CtD-8Qzoco-8Qzo6U-4ayh5W-5xnfwM-3ahGQv-ffDEDm-BzyB2B-7e9bKZ-AC367L-dhJw92-ybwhs-6K9AGp-6K9AvT"),
        author: String::from("Salim Virji"),
        license: String::from("CC BY-SA 2.0"),
        license_url: String::from("https://creativecommons.org/licenses/by-sa/2.0/"),
    };

    let image_json = serde_json::to_string(&image).unwrap();

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

            block_on(sender.send(sse::Data::new(image_json.clone()).event("image"))).unwrap();
            thread::sleep(time::Duration::from_millis(500));
        }

        block_on(sender.send(sse::Data::new("").event("close"))).unwrap();
    });

    stream
        .with_keep_alive(time::Duration::from_secs(1))
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
        let cors = Cors::default()
            .allowed_origin("https://www.planting.life")
            .allowed_origin("https://planting.life")
            .allowed_methods(vec!["GET"]);

        App::new()
            .wrap(cors)
            .service(fetch_entries_handler)
            .service(fetch_entries_handler_mock)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
