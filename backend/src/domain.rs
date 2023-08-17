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

    #[serde(skip_serializing)]
    pub shades: Vec<Shade>,

    #[serde(skip_serializing)]
    pub moistures: Vec<Moisture>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bloom: Option<String>,

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
            done_loading: false,
            highlights: vec![],
        }
    }
    // Merges two plants, prioritizing "other" but never overriding Some with None
    pub fn merge(&self, other: &Plant) -> Plant {
        //TODO: Can I write this concisely with fewer clones?
        Plant {
            id: other.id.or(self.id),
            common: self.common.clone(),
            scientific: self.scientific.clone(),
            moistures: other.moistures.clone(),
            shades: other.shades.clone(),
            bloom: other.bloom.clone().or(self.bloom.clone()),
            image: other.image.clone().or(self.image.clone()),
            pollinator_rating: other.pollinator_rating.or(self.pollinator_rating),
            bird_rating: other.bird_rating.or(self.bird_rating),
            spread_rating: other.spread_rating.or(self.spread_rating),
            deer_resistance_rating: other.deer_resistance_rating.or(self.deer_resistance_rating),
            usda_source: other.usda_source.clone().or(self.usda_source.clone()),
            wiki_source: other.wiki_source.clone().or(self.wiki_source.clone()),
            height: other.height.clone().or(self.height.clone()),
            spread: other.spread.clone().or(self.spread.clone()),
            highlights: other.highlights.clone(),
            done_loading: false,
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

impl Moisture {
    pub fn description(&self) -> &str {
        match self {
            Moisture::None => "low moisture",
            Moisture::Some => "medium moisture",
            Moisture::Lots => "high moisture",
        }
    }
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

    /// A description for this garden, provided by a user
    pub description: String,

    /// The zipcode this garden was created in
    pub zipcode: String,

    /// The name of the region this garden was created in
    pub region_name: Option<String>,

    /// Shade condition this Garden will thrive in
    pub shade: Shade,

    /// Moisture condition this Garden will thrive in
    pub moisture: Moisture,

    /// Is this instance read only in the UI?
    pub read_only: bool,
}

impl Garden {
    /// Creates a Garden without plants or region_name
    pub fn empty(
        name: String,
        description: String,
        zipcode: String,
        shade: Shade,
        moisture: Moisture,
    ) -> Self {
        Self {
            name,
            description,
            zipcode,
            shade,
            moisture,
            plants: vec![],
            region_name: None,
            read_only: true,
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
