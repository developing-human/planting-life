use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::env;

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
    let mut entries = vec![
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
    ];

    let mut entries =
        native_plants::fetch_entries(&api_key, &payload.zip, &payload.shade, &payload.moisture);

    for entry in entries.iter_mut() {
        entry.image_url = get_image_link(&entry.scientific);
    }

    HttpResponse::Ok().json(entries)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(fetch_entries_handler))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

fn old_main() {
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
