use crate::domain::*;
use mysql_async::{prelude::FromRow, FromRowError};
use std::str::FromStr;

impl FromRow for Nursery {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError>
    where
        Self: Sized,
    {
        let (miles, name, url, address, city, state, zip) = mysql_async::from_row_opt(row)?;
        Ok(Nursery {
            name,
            url,
            address,
            city,
            state,
            zip,
            miles,
            map_url: None,
        })
    }
}

impl FromRow for Plant {
    fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, FromRowError>
    where
        Self: Sized,
    {
        // Required fields use take + unwrap (to get value)
        // Optional fields use take_opt + unwrap + ok (to get option)
        let id = row.take("id").unwrap();
        let scientific: String = row.take("scientific_name").unwrap();
        let common = row.take("common_name").unwrap();
        let bloom = row.take("bloom").unwrap();
        let pollinator_rating = row.take_opt("pollinator_rating").unwrap().ok();
        let bird_rating = row.take_opt("bird_rating").unwrap().ok();
        let spread_rating = row.take_opt("spread_rating").unwrap().ok();
        let deer_resistance_rating = row.take_opt("deer_resistance_rating").unwrap().ok();
        let img_id = row.take_opt("image_id").unwrap().ok();
        let title = row.take_opt("title").unwrap().ok();
        let card_url = row.take_opt("card_url").unwrap().ok();
        let original_url = row.take_opt("original_url").unwrap().ok();
        let author = row.take_opt("author").unwrap().ok();
        let license = row.take_opt("license").unwrap().ok();
        let usda_source: Option<String> = row.take_opt("usda_source").unwrap().ok();
        let wiki_source: Option<String> = row.take_opt("wiki_source").unwrap().ok();
        let height: Option<String> = row.take_opt("height").unwrap().ok();
        let spread: Option<String> = row.take_opt("spread").unwrap().ok();

        // These are comma separted, ex: "Some,Lots"
        // This parses them into their respective vectors of enums
        let moistures: Vec<Moisture> = row
            .take_opt("moistures")
            .unwrap()
            .ok()
            .map(|s: String| s.split(',').map(str::parse).map(|r| r.unwrap()).collect())
            .unwrap_or_else(Vec::new);
        let shades: Vec<Shade> = row
            .take_opt("shades")
            .unwrap()
            .ok()
            .map(|s: String| s.split(',').map(str::parse).map(|r| r.unwrap()).collect())
            .unwrap_or_else(Vec::new);

        Ok(Plant {
            id: Some(id),
            scientific,
            common,
            bloom,
            height,
            spread,
            moistures,
            shades,
            pollinator_rating,
            bird_rating,
            spread_rating,
            deer_resistance_rating,
            usda_source,
            wiki_source,
            image: img_id.map(|_| {
                let license: String = license.unwrap();
                let title: String = title.unwrap();
                let card_url: String = card_url.unwrap();
                let original_url: String = original_url.unwrap();
                let author: String = author.unwrap();

                Image {
                    id: img_id,
                    title,
                    card_url,
                    original_url,
                    author,
                    license_url: Image::get_license_url(&license).unwrap(),
                    license,
                }
            }),
            highlights: vec![],
            done_loading: false,
        })
    }
}

impl FromRow for Garden {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError>
    where
        Self: Sized,
    {
        let (name, zipcode, region_name, shade, moisture, read_only) =
            mysql_async::from_row_opt(row)?;

        let zipcode: usize = zipcode;
        let zipcode = format!("{zipcode:05}");

        let moisture: String = moisture;
        let moisture =
            Moisture::from_str(&moisture).expect("gardens.moisture should have valid values");

        let shade: String = shade;
        let shade = Shade::from_str(&shade).expect("gardens.shade should have valid values");
        Ok(Garden {
            name,
            zipcode,
            region_name,
            shade,
            moisture,
            read_only,
            plants: vec![],
        })
    }
}
