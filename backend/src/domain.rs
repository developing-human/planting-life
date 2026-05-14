use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Plant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<usize>,
    pub common: String,
    pub scientific: String,
    pub shades: Vec<Shade>,
    pub moistures: Vec<Moisture>,

    #[serde(skip_serializing)]
    pub habits: Vec<Habit>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bloom: Option<String>,

    pub bloom_months: Vec<Month>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,

    #[serde(skip_serializing)]
    pub pollinator_rating: Option<u8>,

    #[serde(skip_serializing)]
    pub bird_rating: Option<u8>,

    #[serde(skip_serializing)]
    pub spread_rating: Option<u8>,

    #[serde(skip_serializing)]
    pub deer_resistance_rating: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usda_source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wiki_source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wildflower_source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub spread: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub highlights: Vec<Highlight>,

    pub done_loading: bool,
}

impl Plant {
    pub fn new(scientific_name: &str, common_name: &str) -> Plant {
        Plant {
            id: None,
            common: common_name.to_string(),
            scientific: scientific_name.to_string(),
            shades: vec![],
            moistures: vec![],
            habits: vec![],
            bloom: None,
            height: None,
            spread: None,
            pollinator_rating: None,
            bird_rating: None,
            spread_rating: None,
            deer_resistance_rating: None,
            image: None,
            usda_source: None,
            wiki_source: None,
            wildflower_source: None,
            highlights: vec![],
            bloom_months: vec![],
            done_loading: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub id: Option<usize>,
    pub title: String,
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
            "CC BY-NC-SA 4.0" => Some("https://creativecommons.org/licenses/by-nc-sa/4.0/"),
            "CC BY-NC 4.0" => Some("https://creativecommons.org/licenses/by-nc/4.0/"),
            "CC BY-NC-ND 4.0" => Some("https://creativecommons.org/licenses/by-nc-nd/4.0/"),
            "CC BY 4.0" => Some("https://creativecommons.org/licenses/by/4.0/"),
            "CC BY-SA 4.0" => Some("https://creativecommons.org/licenses/by-sa/4.0/"),
            "CC BY-ND 4.0" => Some("https://creativecommons.org/licenses/by-nd/4.0/"),
            "No known copyright restrictions" => Some("https://www.flickr.com/commons/usage/"),
            "US Government Work" => Some("http://www.usa.gov/copyright.shtml"),
            "CC0" => Some("https://creativecommons.org/publicdomain/zero/1.0/"),
            "Public Domain Mark 1.0" => Some("https://creativecommons.org/publicdomain/mark/1.0/"),
            _ => None,
        };

        url.map(|u| u.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Shade {
    #[serde(rename = "Full Sun")]
    None,
    #[serde(rename = "Partial Shade")]
    Some,
    #[serde(rename = "Full Shade")]
    Lots,
}

impl Display for Shade {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Shade {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "None" => Ok(Shade::None),
            "Some" => Ok(Shade::Some),
            "Lots" => Ok(Shade::Lots),
            _ => Err(anyhow!("can't create Shade from {s}")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Moisture {
    #[serde(rename = "Low")]
    None,
    #[serde(rename = "Medium")]
    Some,
    #[serde(rename = "High")]
    Lots,
}

impl Display for Moisture {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Moisture {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "None" => Ok(Moisture::None),
            "Some" => Ok(Moisture::Some),
            "Lots" => Ok(Moisture::Lots),
            _ => Err(anyhow!("can't create Moisture from {s}")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Habit {
    FlowerOrHerb,
    Grass,
    Shrub,
    Tree,
    Vine,
}

impl Display for Habit {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Habit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "FlowerOrHerb" => Ok(Habit::FlowerOrHerb),
            "Grass" => Ok(Habit::Grass),
            "Shrub" => Ok(Habit::Shrub),
            "Tree" => Ok(Habit::Tree),
            "Vine" => Ok(Habit::Vine),
            _ => Err(anyhow!("can't create Habit from {s}")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Month {
    #[serde(rename = "Jan")]
    January,
    #[serde(rename = "Feb")]
    February,
    #[serde(rename = "Mar")]
    March,
    #[serde(rename = "Apr")]
    April,
    #[serde(rename = "May")]
    May,
    #[serde(rename = "Jun")]
    June,
    #[serde(rename = "Jul")]
    July,
    #[serde(rename = "Aug")]
    August,
    #[serde(rename = "Sep")]
    September,
    #[serde(rename = "Oct")]
    October,
    #[serde(rename = "Nov")]
    November,
    #[serde(rename = "Dec")]
    December,
}

impl Display for Month {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Month {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "Jan" => Ok(Month::January),
            "Feb" => Ok(Month::February),
            "Mar" => Ok(Month::March),
            "Apr" => Ok(Month::April),
            "May" => Ok(Month::May),
            "Jun" => Ok(Month::June),
            "Jul" => Ok(Month::July),
            "Aug" => Ok(Month::August),
            "Sep" => Ok(Month::September),
            "Oct" => Ok(Month::October),
            "Nov" => Ok(Month::November),
            "Dec" => Ok(Month::December),
            _ => Err(anyhow!("can't create Month from {s}")),
        }
    }
}

pub struct Conditions {
    pub shade: Vec<Shade>,
    pub moisture: Vec<Moisture>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Highlight {
    pub label: String,
    pub category: HighlightCategory,

    #[serde(skip_serializing)]
    pub priority: usize,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum HighlightCategory {
    #[serde(rename = "great")]
    Great,
    #[serde(rename = "good")]
    Good,
    #[serde(rename = "bad")]
    Bad,
    #[serde(rename = "worse")]
    Worse,
}

/// A named collection of plants, which knows where it is native and the conditions
/// it will thrive in.
#[derive(Serialize)]
pub struct Garden {
    /// The plants in this garden
    pub plants: Vec<Plant>,

    /// A short name for this garden, defaulted to something reasonable but changable
    pub name: String,

    /// The zipcode this garden was created in
    pub zipcode: String,

    /// The name of the region this garden was created in
    pub region_name: Option<String>,

    /// Shade condition this Garden will thrive in
    pub shade: Shade,

    /// Moisture condition this Garden will thrive in
    pub moisture: Moisture,

    /// An identifier which allows read-only access to this Garden
    pub read_id: Option<String>,

    /// An identifier which allows this Garden to be updated
    pub write_id: Option<String>,

    /// The Garden's latitude, if known
    pub latitude: Option<f64>,

    /// The Garden's longitude, if known
    pub longitude: Option<f64>,
}

impl Garden {
    /// Creates a Garden without plants or region_name
    pub fn empty(name: String, zipcode: String, shade: Shade, moisture: Moisture) -> Self {
        Self {
            name,
            zipcode,
            shade,
            moisture,
            plants: vec![],
            region_name: None,
            read_id: None,
            write_id: None,
            latitude: None,
            longitude: None,
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
