use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use futures::executor::block_on;
use futures::join;
use futures::stream::{Stream, StreamExt};
use openai::NativePlant;
use serde::{self, Deserialize, Serialize};
use std::boxed::Box;
use std::env;
use std::pin::Pin;
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
async fn fetch_plants_handler(web::Query(payload): web::Query<FetchRequest>) -> impl Responder {
    println!("Received request: {:#?}", payload);

    let (sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(10);

    // The real work is done in a new thread so the connection to the front end can open.
    actix_web::rt::spawn(async move {
        let mut plants = get_plant_stream(payload).await;
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
                );
            });

            handles.push(handle);
        }

        // Wait for all inner tasks to finish before closing the stream
        // This lets any outstanding data be written back to the client
        for handle in handles {
            handle.await.unwrap_or_default();
        }

        close_stream(&sender).await;
    });

    stream
        .with_keep_alive(time::Duration::from_secs(1))
        .customize()
        .insert_header(("X-Accel-Buffering", "no"))
}

async fn get_plant_stream(payload: FetchRequest) -> Pin<Box<impl Stream<Item = NativePlant>>> {
    let openai_api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");

    let plants = openai::stream_plants(
        &openai_api_key,
        &payload.zip,
        payload.shade.description(),
        payload.moisture.description(),
    )
    .await;

    // I don't yet understand why this is needed.  But it tells the plants
    // not to move in memory during async work.
    Box::pin(plants)
}

async fn send_plant(sender: &Sender, plant: &NativePlant) {
    let json = serde_json::to_string(&plant).expect("plant should serialize");

    sender
        .send(sse::Data::new(json))
        .await
        .expect("plant should send");
}

async fn fetch_and_send_image(sender: &Sender, plant: &NativePlant) {
    let flickr_api_key = env::var("FLICKR_API_KEY").expect("Must define $FLICKR_API_KEY");

    if let Some(image) = flickr::get_image(&plant.scientific, &plant.common, &flickr_api_key).await
    {
        let image_json = serde_json::to_string(&image).expect("image should serialize");
        sender
            .send(sse::Data::new(image_json).event("image"))
            .await
            .expect("image should send");
    }
}

async fn fetch_and_send_description(sender: &Sender, plant: &NativePlant) {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let description_stream = openai::fetch_description(&api_key, &plant.scientific).await;

    let mut description_stream = Box::pin(description_stream);
    while let Some(description_delta) = description_stream.next().await {
        let payload = format!(
            r#"{{"scientificName": "{}", "descriptionDelta": "{}"}}"#,
            plant.scientific, description_delta
        );
        sender
            .send(sse::Data::new(payload).event("descriptionDelta"))
            .await
            .expect("description delta should send");
    }
}

async fn close_stream(sender: &Sender) {
    sender
        .send(sse::Data::new("").event("close"))
        .await
        .expect("stream should close");
}

#[get("/plants_mock")]
async fn fetch_plants_handler_mock(
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
        for mut plant in build_mock_plants() {
            // Remove image to let it load a moment later.
            let image_url = plant.image_url;
            plant.image_url = None;

            let plant_json = serde_json::to_string(&plant).unwrap();

            block_on(sender.send(sse::Data::new(plant_json))).unwrap();
            thread::sleep(time::Duration::from_millis(250));

            block_on(
                sender.send(
                    sse::Data::new(format!("{}::{}", plant.scientific, image_url.unwrap()))
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

fn build_mock_plants() -> impl Iterator<Item = NativePlant> {
    vec![
        NativePlant {
            common: "Wild Columbine".to_string(),
            scientific: "Aquilegia canadensis".to_string(),
            bloom: "Spring to early summer".to_string(),
            description: Some("This plant is a favorite of hummingbirds and supports the Columbine Duskywing butterfly caterpillar.".to_string()),
            image_url: Some("https://live.staticflickr.com/5031/7238526710_80bf103077_q.jpg".to_string()),
        },
        NativePlant {
            common: "Swamp Milkweed".to_string(),
            scientific: "Asclepias incarnata".to_string(),
            bloom: "Summer".to_string(),
            description: Some("This plant is a favorite of hummingbirds and supports the Columbine Duskywing butterfly caterpillar.".to_string()),
            image_url: Some("https://live.staticflickr.com/3126/3147197425_4e9ac1e2ca_q.jpg".to_string()),
        },
        NativePlant {
            common: "Joe Pye Weed".to_string(),
            scientific: "Eutrochium purpureum".to_string(),
            bloom: "Late summer to fall".to_string(),
            description: Some("This plant is a favorite of hummingbirds and supports the Columbine Duskywing butterfly caterpillar.".to_string()),
            image_url: Some("https://live.staticflickr.com/3862/15215414361_9f659f6f52_q.jpg".to_string()),
        },
        NativePlant {
            common: "Blue Flag Iris".to_string(),
            scientific: "Iris versicolor".to_string(),
            bloom: "Late spring to early summer".to_string(),
            description: Some("This plant is a favorite of hummingbirds and supports the Columbine Duskywing butterfly caterpillar.".to_string()),
            image_url: Some("https://live.staticflickr.com/65535/50623901946_1c37f69ccd_q.jpg".to_string()),
        },
        NativePlant {
            common: "Cardinal Flower".to_string(),
            scientific: "Lobelia cardinalis".to_string(),
            bloom: "Late summer to early fall".to_string(),
            description: Some("This plant is a favorite of hummingbirds and supports the Columbine Duskywing butterfly caterpillar.".to_string()),
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
            .service(fetch_plants_handler)
            .service(fetch_plants_handler_mock)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
