use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use futures::executor::block_on;
use native_plants::stream_entries;
use native_plants::NativePlantEntry;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::env;
use std::thread;
use std::time;

#[derive(Serialize, Deserialize, Debug)]
struct FetchRequest {
    zip: String,
    shade: String,
    moisture: String,
}

#[get("/plants")]
async fn fetch_entries_handler(web::Query(payload): web::Query<FetchRequest>) -> impl Responder {
    println!("Received request: {:#?}", payload);
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");

    //TODO: Remove me
    let mut entries = build_fake_plants();

    let mut entries =
        native_plants::fetch_entries(&api_key, &payload.zip, &payload.shade, &payload.moisture);

    for entry in entries.iter_mut() {
        entry.image_url = get_image_link(&entry.scientific);
    }

    HttpResponse::Ok().json(entries)
}

#[get("/plants_sse")]
async fn fetch_entries_handler_sse(
    web::Query(payload): web::Query<FetchRequest>,
) -> impl Responder {
    println!("Received request: {:#?}", payload);

    let (sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(1);

    thread::spawn(move || {
        //TODO: Remove me
        let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
        let mut entries = build_fake_plants();
        let entries = native_plants::stream_entries(
            &api_key,
            &payload.zip,
            &payload.shade,
            &payload.moisture,
        );

        for mut entry in entries {
            entry.image_url = get_image_link(&entry.scientific);

            let entry_json = serde_json::to_string(&entry).unwrap();
            //TODO: Can I get rid of the "block on"?
            //      I think I need this thread to be async?

            block_on(sender.send(sse::Data::new(entry_json))).unwrap();

            //TODO: Remove delay
            thread::sleep(time::Duration::from_secs(1));
        }

        block_on(sender.send(sse::Data::new("").event("close"))).unwrap();
    });

    stream.with_keep_alive(time::Duration::from_secs(1))
}

fn build_fake_plants() -> Vec<NativePlantEntry> {
    vec![
        native_plants::NativePlantEntry {
            common: "California Wild Strawberry".to_string(),
            scientific: "Fragaria californica".to_string(),
            description: "the kind of long description".to_string(),
            image_url: None,
        },
        native_plants::NativePlantEntry {
            common: "Evergreen Huckleberry".to_string(),
            scientific: "Vaccinium ovatum".to_string(),
            description: "another kind of long description".to_string(),
            image_url: None,
        },
        native_plants::NativePlantEntry {
            common: "Evergreen Huckleberry".to_string(),
            scientific: "Vaccinium ovatum".to_string(),
            description: "another kind of long description".to_string(),
            image_url: None,
        },
        native_plants::NativePlantEntry {
            common: "Evergreen Huckleberry".to_string(),
            scientific: "Vaccinium ovatum".to_string(),
            description: "another kind of long description".to_string(),
            image_url: None,
        },
        native_plants::NativePlantEntry {
            common: "Evergreen Huckleberry".to_string(),
            scientific: "Vaccinium ovatum".to_string(),
            description: "another kind of long description".to_string(),
            image_url: None,
        },
    ]
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            //TODO: Don't do this in prod... but it lets me skip using the
            //      React proxy server which causes issues with streaming events
            .wrap(Cors::permissive())
            .service(fetch_entries_handler)
            .service(fetch_entries_handler_sse)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn cli_main() {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let entries = native_plants::fetch_entries(&api_key, "43081", "partial shade", "wet soil");

    for (index, entry) in entries.iter().enumerate() {
        println!("{} ({})", entry.common, entry.scientific);
        println!("{}", entry.description);

        if index != entries.len() - 1 {
            println!();
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ResultItem {
    media: Vec<MediaItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MediaItem {
    identifier: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    results: Vec<ResultItem>,
}

fn get_image_link(scientific_name: &str) -> Option<String> {
    // Recent, permissive license, human observation
    let fetch_url = format!("https://api.gbif.org/v1/occurrence/search?scientificName={}&mediaType=StillImage&limit=20&license=CC_0_1_0&basisOfRecord=HUMAN_OBSERVATION&year=2015,2023", scientific_name);

    let client = reqwest::blocking::Client::new();
    let response = client.get(fetch_url).send().expect("Error calling model");

    let status = response.status();
    let response_body = response
        .text()
        .expect("Error extracting body from response");

    if status != StatusCode::OK {
        eprintln!("Error from model endpoint: {response_body}");
        std::process::exit(1);
    }

    let parsed_response: Payload =
        serde_json::from_str(&response_body).expect("Error parsing response");

    for result in parsed_response.results {
        for media in result.media {
            if media.identifier.is_some() {
                return media.identifier;
            }
        }
    }

    None
}
