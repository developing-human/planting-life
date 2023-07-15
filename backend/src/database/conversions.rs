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
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError>
    where
        Self: Sized,
    {
        let (
            id,
            scientific,
            common,
            bloom,
            description,
            img_id,
            title,
            card_url,
            original_url,
            author,
            license,
        ) = mysql_async::from_row_opt(row)?;

        // Everything related to the image is optional because the image may not exist
        // But if img_id is present, everything else is required.  Hence the unwraps.
        let img_id: Option<usize> = img_id;
        let license: Option<String> = license;
        let scientific: String = scientific;

        Ok(Plant {
            id: Some(id),
            scientific: scientific.to_string(),
            common,
            description,
            bloom,
            image: img_id.map(|_| {
                let license = license.unwrap();

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
