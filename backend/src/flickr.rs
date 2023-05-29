use futures::join;
use reqwest::StatusCode;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::warn;

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
    description: ImageSearchPhotoDescription,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageSearchPhotoDescription {
    _content: String,
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

pub async fn get_image(scientific_name: &str, common_name: &str, api_key: &str) -> Option<Image> {
    // Remove "spp." from the end if it exists, this is an abbreviation for "species".
    let truncated_scientific_name = &scientific_name.replace(" spp.", "");

    // Run searches for both blooming and non-blooming concurrently.
    // This is a little aggressive since the latter won't be used if the former finds results
    let search_term = format!("{} blooming", scientific_name);
    let blooming_search = image_search(&search_term, api_key);
    let non_blooming_search = image_search(scientific_name, api_key);
    let (blooming_result, non_blooming_result) = join!(blooming_search, non_blooming_search);

    // First, look for this plant in bloom
    if let Some(response) = blooming_result {
        if let Some(image) = find_best_photo(
            response,
            scientific_name,
            truncated_scientific_name,
            common_name,
        ) {
            return Some(image);
        }
    }

    // If it can't be found in bloom, look for any other image of it
    if let Some(response) = non_blooming_result {
        if let Some(image) = find_best_photo(
            response,
            scientific_name,
            truncated_scientific_name,
            common_name,
        ) {
            return Some(image);
        }
    }

    None // No image to show :(
}

#[tracing::instrument]
async fn image_search(search_term: &str, api_key: &str) -> Option<ImageSearchResponse> {
    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(Duration::from_millis(100), Duration::from_millis(500))
        .build_with_max_retries(4);

    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let response = client
        .get("https://api.flickr.com/services/rest")
        .timeout(Duration::from_millis(2_000)) // Typically 350-500ms, sometimes ~1s
        .query(&[
            ("method", "flickr.photos.search"),
            ("api_key", api_key),
            ("text", search_term),
            ("media", "photos"),
            ("format", "json"),
            ("nojsoncallback", "1"),
            ("extras", "views,url_q,license,owner_name,description"),
            ("min_upload_date", "2015-01-01"),
            ("sort", "relevance"),
            // This is everything except "All Rights Reserved"
            // docs here: https://www.flickr.com/services/api/flickr.photos.licenses.getInfo.html
            ("license", "1,2,3,4,5,6,7,8,9,10"),
        ])
        .send()
        .await;

    let response = match response {
        Ok(r) => r,
        Err(_) => {
            warn!("Error fetching response for: {search_term}");
            return None;
        }
    };

    let status = response.status();
    let response_body = response.text().await;

    let response_body = match response_body {
        Ok(rb) => rb,
        Err(_) => {
            warn!("Error fetching response body for: {search_term}");
            return None;
        }
    };

    if status != StatusCode::OK {
        warn!("Error from model endpoint: {response_body}");
        return None;
    }

    let parsed_response: serde_json::Result<ImageSearchResponse> =
        serde_json::from_str(&response_body);
    match parsed_response {
        Ok(response) => Some(response),
        Err(_) => None,
    }
}

fn find_best_photo(
    response: ImageSearchResponse,
    scientific_name: &str,
    truncated_scientific_name: &str,
    common_name: &str,
) -> Option<Image> {
    let mut highest_title_views = -1;
    let mut highest_title = None;
    let mut highest_description_views = -1;
    let mut highest_description = None;
    let scientific_name_lc = truncated_scientific_name.to_lowercase();
    let common_name_lc = common_name.to_lowercase();

    // Search for the most viewed photo which has the scientific or common name in the title
    // In case none are found, also track the most viewed overall
    'photo: for photo in response.photos.photo.iter() {
        let photo_views = match photo.views.parse::<i32>() {
            Ok(views) => views,
            Err(_) => {
                warn!("Could not parse {} as i32", photo.views);
                continue;
            }
        };

        let title_lc = photo.title.to_lowercase();
        let description_lc = photo.description._content.to_lowercase();

        // Certain words in the description mean we should ignore this, usually because
        // they are hand drawn rather than photos
        for banned_word in &vec!["drawn", "illustration", "dried wildflowers", "illustrated"] {
            if description_lc.contains(banned_word) {
                continue 'photo;
            }
        }

        if (title_lc.contains(&scientific_name_lc) || title_lc.contains(&common_name_lc))
            && photo_views > highest_title_views
        {
            highest_title_views = photo_views;
            highest_title = Some(photo);
        }

        if (description_lc.contains(&scientific_name_lc)
            || description_lc.contains(&common_name_lc))
            && photo_views > highest_description_views
        {
            highest_description_views = photo_views;
            highest_description = Some(photo);
        }
    }

    // If any had scientific or common name, return most viewed of those
    if let Some(photo) = highest_title {
        return Image::from_photo(photo, scientific_name);
    }

    // If any had scientific or common name, return most viewed of those
    if let Some(photo) = highest_description {
        return Image::from_photo(photo, scientific_name);
    }

    // Don't try returning photos without a common/scientific name match
    // That tends to choose popular photos that are of a different plant.

    None // No image found :(
}
