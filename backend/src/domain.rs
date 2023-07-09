use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NativePlant {
    pub common: String,
    pub scientific: String,
    pub bloom: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub scientific_name: String,
    pub title: String,
    pub thumbnail_url: String,
    pub card_url: String,
    pub original_url: String,
    pub author: String,
    pub license: String,
    pub license_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Shade {
    #[serde(rename = "Full Shade")]
    Full,
    #[serde(rename = "Partial Shade")]
    Partial,
    #[serde(rename = "Full Sun")]
    No,
}

impl Shade {
    pub fn description(&self) -> &str {
        match self {
            Shade::Full => "full shade",
            Shade::Partial => "partial shade",
            Shade::No => "full sun",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Moisture {
    Low,
    Medium,
    High,
}

impl Moisture {
    pub fn description(&self) -> &str {
        match self {
            Moisture::Low => "dry soil",
            Moisture::Medium => "moderately wet soil",
            Moisture::High => "wet soil",
        }
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
