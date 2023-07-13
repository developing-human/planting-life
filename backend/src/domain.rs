use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NativePlant {
    pub id: Option<usize>,
    pub common: String,
    pub scientific: String,
    pub bloom: Option<String>,
    pub description: Option<String>,
    //pub image_url: Option<String>,
    pub image: Option<Image>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub id: Option<usize>,
    pub scientific_name: String, //TODO: Only here b/c its needed on front end
    pub title: String,
    //pub thumbnail_url: String,
    pub card_url: String,
    pub original_url: String,
    pub author: String,
    pub license: String,
    pub license_url: String,
}

impl Image {
    pub fn get_license_url(license_id: &str) -> Option<String> {
        let url = match license_id {
            "CC BY-NC-SA 2.0" => Some("https://creativecommons.org/licenses/by-nc-sa/2.0/"),
            "CC BY-NC 2.0" => Some("https://creativecommons.org/licenses/by-nc/2.0/"),
            "CC BY-NC-ND 2.0" => Some("https://creativecommons.org/licenses/by-nc-nd/2.0/"),
            "CC BY 2.0" => Some("https://creativecommons.org/licenses/by/2.0/"),
            "CC BY-SA 2.0" => Some("https://creativecommons.org/licenses/by-sa/2.0/"),
            "CC BY-ND 2.0" => Some("https://creativecommons.org/licenses/by-nd/2.0/"),
            "No known copyright restrictions" => Some("https://www.flickr.com/commons/usage/"),
            "US Government Work" => Some("http://www.usa.gov/copyright.shtml"),
            "CC0" => Some("https://creativecommons.org/publicdomain/zero/1.0/"),
            "Public Domain Mark 1.0" => Some("https://creativecommons.org/publicdomain/mark/1.0/"),
            _ => None,
        };

        url.map(|u| u.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Shade {
    #[serde(rename = "Full Sun")]
    None,
    #[serde(rename = "Partial Shade")]
    Some,
    #[serde(rename = "Full Shade")]
    Lots,
}

impl Shade {
    pub fn description(&self) -> &str {
        match self {
            Shade::None => "full sun",
            Shade::Some => "partial shade",
            Shade::Lots => "full shade",
        }
    }
}

impl Display for Shade {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Moisture {
    #[serde(rename = "Low")]
    None,
    #[serde(rename = "Medium")]
    Some,
    #[serde(rename = "High")]
    Lots,
}

impl Moisture {
    pub fn description(&self) -> &str {
        match self {
            Moisture::None => "dry soil",
            Moisture::Some => "moderately wet soil",
            Moisture::Lots => "wet soil",
        }
    }
}

impl Display for Moisture {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nursery {
    pub name: String,
    pub url: Option<String>,
    pub map_url: Option<String>,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: usize,
    pub miles: usize,
}

impl Nursery {
    pub fn build_default_map_url(&self) -> String {
        // Pad the zip code to five digits, using zeros.
        let zip = format!("{:05}", self.zip);

        let query = format!("{} near {}", self.name, zip);
        let query = query.replace(' ', "+");
        let url = format!("https://www.google.com/maps/search/?api=1&query={query}");

        url
    }
}
