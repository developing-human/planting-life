use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use futures::executor::block_on;
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

    let (sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(10);

    thread::spawn(move || {
        let openai_api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
        let flickr_api_key = env::var("FLICKR_API_KEY").expect("Must define $OPENAI_API_KEY");

        let entries = native_plants::stream_entries(
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

            if let Some(image_url) = get_image_link(&entry.scientific, &flickr_api_key) {
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
}

#[derive(Serialize, Deserialize, Debug)]
struct FlickrResponse {
    photos: FlickrResponsePhotos,
}

#[derive(Serialize, Deserialize, Debug)]
struct FlickrResponsePhotos {
    photo: Vec<FlickrResponsePhoto>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FlickrResponsePhoto {
    url_q: String,
    title: String,
    views: String,
}

fn get_image_link(scientific_name: &str, api_key: &str) -> Option<String> {
    // First, look for this plant in bloom.
    let search_term = format!("{} blooming", scientific_name);
    if let Some(response) = call_flickr(&search_term, api_key) {
        if let Some(img_url) = find_best_photo(response, scientific_name) {
            return Some(img_url);
        }
    }

    // If it can't be found in bloom, look for any other image of it
    if let Some(response) = call_flickr(scientific_name, api_key) {
        if let Some(img_url) = find_best_photo(response, scientific_name) {
            return Some(img_url);
        }
    }

    None // No image to show :(
}

fn call_flickr(search_term: &str, api_key: &str) -> Option<FlickrResponse> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.flickr.com/services/rest")
        .query(&[
            ("method", "flickr.photos.search"),
            ("api_key", api_key),
            ("text", search_term),
            ("format", "json"),
            ("nojsoncallback", "1"),
            ("extras", "views,url_q"),
            ("sort", "relevance"),
            // This is everything except "All Rights Reserved"
            // docs here: https://www.flickr.com/services/api/flickr.photos.licenses.getInfo.html
            ("license", "1,2,3,4,5,6,7,8,9,10"),
        ])
        .send()
        .expect("Error fetching image");

    let status = response.status();
    let response_body = response
        .text()
        .expect("Error extracting body from response");

    if status != StatusCode::OK {
        eprintln!("Error from model endpoint: {response_body}");
        return None;
    }

    let parsed_response: serde_json::Result<FlickrResponse> = serde_json::from_str(&response_body);
    match parsed_response {
        Ok(response) => Some(response),
        Err(_) => None,
    }
}

fn find_best_photo(response: FlickrResponse, scientific_name: &str) -> Option<String> {
    let mut highest_match_views = -1;
    let mut highest_match = None;
    let mut highest_views = -1;
    let mut highest = None;
    let scientific_name_lc = scientific_name.to_lowercase();

    // Search for the most viewed photo which has the scientific name the title
    // In case none are found, also track the most viewed overall
    for photo in response.photos.photo.iter() {
        let photo_views = photo.views.parse::<i32>().unwrap();
        if photo_views > highest_views {
            highest_views = photo_views;
            highest = Some(photo);
        }

        let title_lc = photo.title.to_lowercase();
        if title_lc.contains(&scientific_name_lc) && photo_views > highest_match_views {
            highest_match_views = photo_views;
            highest_match = Some(photo);
        }
    }

    // If any had scientific name, return most viewed of those
    if let Some(photo) = highest_match {
        return Some(String::from(&photo.url_q));
    }

    // Otherwise, return most viewed overall
    if let Some(photo) = highest {
        return Some(String::from(&photo.url_q));
    }

    None
}

fn build_mock_plants() -> impl Iterator<Item = NativePlantEntry> {
    vec![
        native_plants::NativePlantEntry {
            common: "Wild Columbine".to_string(),
            scientific: "Aquilegia canadensis".to_string(),
            bloom: "Spring to early summer".to_string(),
            description: "This plant is a favorite of hummingbirds and supports the Columbine Duskywing butterfly caterpillar.".to_string(),
            image_url: Some("https://live.staticflickr.com/5031/7238526710_80bf103077_q.jpg".to_string()),
        },
        native_plants::NativePlantEntry {
            common: "Swamp Milkweed".to_string(),
            scientific: "Asclepias incarnata".to_string(),
            bloom: "Summer".to_string(),
            description: "This plant is a host for the Monarch butterfly caterpillar and supports many other pollinators.".to_string(),
            image_url: Some("https://live.staticflickr.com/3126/3147197425_4e9ac1e2ca_q.jpg".to_string()),
        },
        native_plants::NativePlantEntry {
            common: "Joe Pye Weed".to_string(),
            scientific: "Eutrochium purpureum".to_string(),
            bloom: "Late summer to fall".to_string(),
            description: "This plant is a favorite of many pollinators, including bees and butterflies.".to_string(),
            image_url: Some("https://live.staticflickr.com/3862/15215414361_9f659f6f52_q.jpg".to_string()),
        },
        native_plants::NativePlantEntry {
            common: "Blue Flag Iris".to_string(),
            scientific: "Iris versicolor".to_string(),
            bloom: "Late spring to early summer".to_string(),
            description: "This plant supports the Baltimore Checkerspot butterfly caterpillar and is a favorite of many pollinators.".to_string(),
            image_url: Some("https://live.staticflickr.com/65535/50623901946_1c37f69ccd_q.jpg".to_string()),
        },
        native_plants::NativePlantEntry {
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
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
