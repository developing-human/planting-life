use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ImageSearchResponse {
    photos: ImageSearchPhotos,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageSearchPhotos {
    photo: Vec<ImageSearchPhoto>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageSearchPhoto {
    id: String,
    owner: String,
    url_q: String,
    views: String,
    title: String,
    license: String,
    ownername: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub scientific_name: String,
    pub title: String,
    pub thumbnail_url: String,
    pub original_url: String,
    pub author: String,
    pub license: String,
    pub license_url: String,
}

impl Image {
    fn from_photo(photo: &ImageSearchPhoto, scientific_name: &str) -> Option<Image> {
        Some(Image {
            scientific_name: String::from(scientific_name),
            title: photo.title.clone(),
            thumbnail_url: photo.url_q.clone(),
            original_url: format!("https://www.flickr.com/photos/{}/{}", photo.owner, photo.id),
            author: photo.ownername.clone(),
            license: get_license_name(&photo.license)?.to_string(),
            license_url: get_license_url(&photo.license)?.to_string(),
        })
    }
}

fn get_license_name(license_id: &str) -> Option<&str> {
    match license_id {
        "1" => Some("CC BY-NC-SA 2.0"),
        "2" => Some("CC BY-NC 2.0"),
        "3" => Some("CC BY-NC-ND 2.0"),
        "4" => Some("CC BY 2.0"),
        "5" => Some("CC BY-SA 2.0"),
        "6" => Some("CC BY-ND 2.0"),
        "7" => Some("No known copyright restrictions"),
        "8" => Some("US Government Work"),
        "9" => Some("CC0"),
        "10" => Some("Public Domain Mark 1.0"),
        _ => None,
    }
}

fn get_license_url(license_id: &str) -> Option<&str> {
    match license_id {
        "1" => Some("https://creativecommons.org/licenses/by-nc-sa/2.0/"),
        "2" => Some("https://creativecommons.org/licenses/by-nc/2.0/"),
        "3" => Some("https://creativecommons.org/licenses/by-nc-nd/2.0/"),
        "4" => Some("https://creativecommons.org/licenses/by/2.0/"),
        "5" => Some("https://creativecommons.org/licenses/by-sa/2.0/"),
        "6" => Some("https://creativecommons.org/licenses/by-nd/2.0/"),
        "7" => Some("https://www.flickr.com/commons/usage/"),
        "8" => Some("http://www.usa.gov/copyright.shtml"),
        "9" => Some("https://creativecommons.org/publicdomain/zero/1.0/"),
        "10" => Some("https://creativecommons.org/publicdomain/mark/1.0/"),
        _ => None,
    }
}

pub fn get_image(scientific_name: &str, api_key: &str) -> Option<Image> {
    // First, look for this plant in bloom
    let search_term = format!("{} blooming", scientific_name);
    if let Some(response) = image_search(&search_term, api_key) {
        if let Some(image) = find_best_photo(response, scientific_name) {
            return Some(image);
        }
    }

    // If it can't be found in bloom, look for any other image of it
    if let Some(response) = image_search(scientific_name, api_key) {
        if let Some(image) = find_best_photo(response, scientific_name) {
            return Some(image);
        }
    }

    None // No image to show :(
}

fn image_search(search_term: &str, api_key: &str) -> Option<ImageSearchResponse> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.flickr.com/services/rest")
        .query(&[
            ("method", "flickr.photos.search"),
            ("api_key", api_key),
            ("text", search_term),
            ("media", "photos"),
            ("format", "json"),
            ("nojsoncallback", "1"),
            ("extras", "views,url_q,license,owner_name"),
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

    let parsed_response: serde_json::Result<ImageSearchResponse> =
        serde_json::from_str(&response_body);
    match parsed_response {
        Ok(response) => Some(response),
        Err(_) => None,
    }
}

fn find_best_photo(response: ImageSearchResponse, scientific_name: &str) -> Option<Image> {
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
        return Image::from_photo(photo, scientific_name);
    }

    // Otherwise, return most viewed overall
    if let Some(photo) = highest {
        return Image::from_photo(photo, scientific_name);
    }

    None // No image found :(
}
