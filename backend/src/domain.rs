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
    pub scientific_name: String, //TODO: Duplicate, remove soon.
    pub title: String,
    //pub thumbnail_url: String,
    pub card_url: String,
    pub original_url: String,
    pub author: String,
    pub license: String,
    pub license_url: String,
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
