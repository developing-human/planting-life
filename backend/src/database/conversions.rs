use crate::domain::*;
use mysql_async::{prelude::FromRow, FromRowError};

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
        let pollinator_reason = row.take_opt("pollinator_reason").unwrap().ok();
        let bird_rating = row.take_opt("bird_rating").unwrap().ok();
        let bird_reason = row.take_opt("bird_reason").unwrap().ok();
        let animal_rating = row.take_opt("animal_rating").unwrap().ok();
        let animal_reason = row.take_opt("animal_reason").unwrap().ok();
        let img_id = row.take_opt("image_id").unwrap().ok();
        let title = row.take_opt("title").unwrap().ok();
        let card_url = row.take_opt("card_url").unwrap().ok();
        let original_url = row.take_opt("original_url").unwrap().ok();
        let author = row.take_opt("author").unwrap().ok();
        let license = row.take_opt("license").unwrap().ok();
        let usda: Option<String> = row.take_opt("usda_source").unwrap().ok();
        let wiki: Option<String> = row.take_opt("wiki_source").unwrap().ok();

        let mut citations = vec![];
        if let Some(usda) = usda {
            citations.push(Citation::create_usda(&usda));
        }
        if let Some(wiki) = wiki {
            citations.push(Citation::create_wikipedia(&wiki));
        }

        Ok(Plant {
            id: Some(id),
            scientific: scientific.clone(),
            common,
            bloom,
            pollinator_rating: pollinator_rating.map(|rating| Rating {
                rating,
                reason: pollinator_reason.unwrap(),
            }),
            bird_rating: bird_rating.map(|rating| Rating {
                rating,
                reason: bird_reason.unwrap(),
            }),
            animal_rating: animal_rating.map(|rating| Rating {
                rating,
                reason: animal_reason.unwrap(),
            }),
            citations,
            image: img_id.map(|_| {
                let license: String = license.unwrap();
                let title: String = title.unwrap();
                let card_url: String = card_url.unwrap();
                let original_url: String = original_url.unwrap();
                let author: String = author.unwrap();

                Image {
                    id: img_id,
                    scientific_name: scientific,
                    title,
                    card_url,
                    original_url,
                    author,
                    license_url: Image::get_license_url(&license).unwrap(),
                    license,
                }
            }),
        })
    }
}
