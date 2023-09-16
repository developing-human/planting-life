use crate::domain::*;
use mysql_async::{
    prelude::{FromRow, FromValue},
    FromRowError, Row,
};
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
        let bloom = take_lenient(&mut row, "bloom");
        let pollinator_rating = take_lenient(&mut row, "pollinator_rating");
        let bird_rating = take_lenient(&mut row, "bird_rating");
        let spread_rating = take_lenient(&mut row, "spread_rating");
        let deer_resistance_rating = take_lenient(&mut row, "deer_resistance_rating");
        let img_id = take_lenient(&mut row, "image_id");
        let title = take_lenient(&mut row, "title");
        let card_url = take_lenient(&mut row, "card_url");
        let original_url = take_lenient(&mut row, "original_url");
        let author = take_lenient(&mut row, "author");
        let license = take_lenient(&mut row, "license");
        let usda_source: Option<String> = take_lenient(&mut row, "usda_source");
        let wiki_source: Option<String> = take_lenient(&mut row, "wiki_source");
        let height: Option<String> = take_lenient(&mut row, "height");
        let spread: Option<String> = take_lenient(&mut row, "spread");

        // These are comma separated, ex: "Some,Lots"
        // This parses them into their respective vectors of enums
        let moistures: Vec<Moisture> = take_lenient(&mut row, "moistures")
            .map(|s: String| s.split(',').map(str::parse).map(|r| r.unwrap()).collect())
            .unwrap_or_else(Vec::new);
        let shades: Vec<Shade> = take_lenient(&mut row, "shades")
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
            images: img_id.map_or(vec![], |_| {
                let license: String = license.unwrap();
                let title: String = title.unwrap();
                let card_url: String = card_url.unwrap();
                let original_url: String = original_url.unwrap();
                let author: String = author.unwrap();

                vec![Image {
                    id: img_id,
                    title,
                    card_url,
                    original_url,
                    author,
                    license_url: Image::get_license_url(&license).unwrap(),
                    license,
                }]
            }),
            highlights: vec![],
            done_loading: false,
        })
    }
}

impl FromRow for Image {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError>
    where
        Self: Sized,
    {
        let (id, title, card_url, original_url, author, license) = mysql_async::from_row_opt(row)?;

        let license: String = license;

        Ok(Image {
            id: Some(id),
            title,
            card_url,
            original_url,
            author,
            license_url: Image::get_license_url(&license).unwrap(),
            license,
        })
    }
}

impl FromRow for Garden {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError>
    where
        Self: Sized,
    {
        let (name, zipcode, region_name, shade, moisture, read_id) =
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
            read_id,
            write_id: None,
            plants: vec![],
        })
    }
}

pub fn take_lenient<T>(row: &mut Row, field: &str) -> Option<T>
where
    T: FromValue,
{
    match row.take_opt(field) {
        Some(Ok(value)) => Some(value),
        Some(Err(_)) => None, //TODO: Maybe log
        None => None,
    }
}
