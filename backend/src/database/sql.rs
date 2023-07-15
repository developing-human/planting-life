use crate::domain::*;
use anyhow::anyhow;
use mysql_async::prelude::*;
use mysql_async::FromRowError;

use super::Database;

pub async fn select_nurseries_by_zip(db: &Database, zip: &str) -> anyhow::Result<Vec<Nursery>> {
    let mut conn = db.get_connection().await?;

    r"
SELECT miles, name, url, address, city, state, n.zipcode
FROM zipcodes_nurseries zn
INNER JOIN nurseries n
  ON n.id = zn.nursery_id
WHERE zn.zipcode = ?
ORDER BY miles ASC"
        .with((zip,))
        .map(&mut conn, |nursery: Nursery| nursery)
        .await
        .map_err(|e| anyhow!("{e}"))
}
pub async fn select_plants_by_zip_moisture_shade(
    db: &Database,
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<Vec<NativePlant>> {
    let mut conn = db.get_connection().await?;

    r"
SELECT p.id, p.scientific_name, p.common_name, p.bloom, p.description, i.id, i.title, i.card_url, i.original_url, i.author, i.license
FROM plants p
INNER JOIN queries_plants qp ON qp.plant_id = p.id
INNER JOIN queries q ON qp.query_id = q.id
INNER JOIN zipcodes z ON z.region_id = q.region_id
LEFT JOIN images i ON i.id = p.image_id
WHERE z.zipcode = ?
  AND q.moisture = ?
  AND q.shade = ?
"
        .with((zip, moisture.to_string(), shade.to_string()))
        .map(&mut conn, |plant: NativePlant| plant)
        .await
        .map_err(|e| anyhow!("{e}"))
}

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

impl FromRow for NativePlant {
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

        Ok(NativePlant {
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
