use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

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

pub fn get_image_url(scientific_name: &str, api_key: &str) -> Option<String> {
    // First, look for this plant in bloom
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

    None // No image found :(
}
