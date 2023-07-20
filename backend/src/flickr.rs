use crate::domain::Image;
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
    url_z: Option<String>,
    height_z: Option<u32>,
    width_z: Option<u32>,
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

impl Image {
    fn from_photo(photo: &ImageSearchPhoto) -> Option<Image> {
        Some(Image {
            id: None,
            title: photo.title.clone(),
            card_url: photo.url_z.clone().unwrap(), // only photos with url_z are chosen
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
        if let Some(image) = find_best_photo(response, truncated_scientific_name, common_name) {
            return Some(image);
        }
    }

    // If it can't be found in bloom, look for any other image of it
    if let Some(response) = non_blooming_result {
        if let Some(image) = find_best_photo(response, truncated_scientific_name, common_name) {
            return Some(image);
        }
    }

    //TODO: Consider searching for common name in image_search as last ditch effort.

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
            ("extras", "views,url_q,url_z,license,owner_name,description"),
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
        Err(e) => {
            warn!("Cannot parse flickr response: {}", e);
            None
        }
    }
}

fn find_best_photo(
    response: ImageSearchResponse,
    truncated_scientific_name: &str,
    common_name: &str,
) -> Option<Image> {
    let scientific_name_lc = truncated_scientific_name.to_lowercase();
    let common_name_lc = common_name.to_lowercase();

    // Filter to valid photos, where valid means:
    //   1. photo.views represents a valid integer
    //   2. The photo id is not blocked (some photos are bad for this use case)
    //   3. The description doesn't have any blocked words which hint at illustrations
    let mut valid_photos = response
        .photos
        .photo
        .into_iter()
        .filter(|photo| photo.url_z.is_some())
        .filter(|photo| match photo.views.parse::<i32>() {
            Ok(_) => true,
            Err(_) => {
                warn!("Could not parse {} as i32", photo.views);
                false
            }
        })
        .filter(|photo| {
            // Filter out images that aren't useful in this context
            // This is more of a stop gap until there's an interface to choose images
            ![
                "37831198204", // educational drawing of carex crinita
                "17332010645", // field of apparently dead goldenrod?
                "43826520262", // too close up of wild ginger
                "41085999240", // too close up of wild ginger
                "26596674001", // too close up of wild ginger
                "37356079394", // too close up of black eyed susan
            ]
            .contains(&photo.id.as_str())
        })
        .filter(|photo| {
            let description_lc = photo.description._content.to_lowercase();
            let title_lc = photo.title.to_lowercase();

            // Certain words in the description mean we should ignore this, usually because
            // they are hand drawn rather than photos
            for blocked_word in &vec!["drawn", "illustration", "dried wildflowers", "illustrated"] {
                if description_lc.contains(blocked_word) {
                    return false;
                }
                if title_lc.contains(blocked_word) {
                    return false;
                }
            }

            true
        })
        .collect::<Vec<ImageSearchPhoto>>();

    // Sort such that the highest priority images come first.
    // Priorities:
    //   1. Title has plant name
    //   2. Is landscape
    //   3. Views
    valid_photos.sort_unstable_by(|a, b| {
        let a_title_lc = a.title.to_lowercase();
        let a_has_name =
            a_title_lc.contains(&scientific_name_lc) || a_title_lc.contains(&common_name_lc);
        let b_title_lc = b.title.to_lowercase();
        let b_has_name =
            b_title_lc.contains(&scientific_name_lc) || b_title_lc.contains(&common_name_lc);

        // Containing the name gives highest priority
        if a_has_name && !b_has_name {
            return std::cmp::Ordering::Less;
        }
        if !a_has_name && b_has_name {
            return std::cmp::Ordering::Greater;
        }

        // Being landscape is next highest priority
        // Unwraps are ok because photos without _z are filtered earlier
        let a_is_landscape = a.width_z.unwrap() >= a.height_z.unwrap();
        let b_is_landscape = b.width_z.unwrap() >= b.height_z.unwrap();
        if a_is_landscape && !b_is_landscape {
            return std::cmp::Ordering::Less;
        }
        if !a_is_landscape && b_is_landscape {
            return std::cmp::Ordering::Greater;
        }

        // Unwrap is ok, invalid views strings are filtered above
        let a_views = a.views.parse::<i32>().unwrap();
        let b_views = b.views.parse::<i32>().unwrap();

        // This is flipped because more views gives higher priority
        // and the highest priority is at the beginning of the list
        b_views.cmp(&a_views)
    });

    valid_photos.first().and_then(Image::from_photo)
}
