use std::collections::HashSet;

use crate::domain::*;
use anyhow::anyhow;
use mysql_async::prelude::*;

use super::Database;

/// Inserts a new Query into the database.  
/// Returns Err if it fails.
pub async fn insert_query(
    db: &Database,
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<usize> {
    let mut conn = db.get_connection().await?;
    let queries_result: Result<Option<usize>, mysql_async::Error> =
        r"INSERT INTO queries (moisture, shade, region_id) VALUES
            (?, ?, (SELECT region_id from zipcodes where zipcode = ?))
            RETURNING id"
            .with((moisture.to_string(), shade.to_string(), zip))
            .first(&mut conn)
            .await;

    match queries_result {
        Ok(Some(id)) => Ok(id),
        Ok(None) => Err(anyhow!(
            "save_query_results saved query but did not receive id"
        )),
        Err(e) => Err(anyhow!(
            "save_query_results insert into queries failed: {}",
            e
        )),
    }
}

/// Inserts into queries_plants.  
/// Returns Err if it fails.
pub async fn insert_query_plants(
    db: &Database,
    query_id: usize,
    plant_ids: HashSet<usize>,
) -> anyhow::Result<()> {
    let mut conn = db.get_connection().await?;

    r"INSERT INTO queries_plants (query_id, plant_id)
            VALUES (:query_id, :plant_id)"
        .with(plant_ids.iter().map(|id| {
            params! {
                "query_id" => query_id,
                "plant_id" => id
            }
        }))
        .batch(&mut conn)
        .await
        .map_err(|e| anyhow!(e))
}

/// Updates one plant.  
/// Returns Err if it fails.
pub async fn update_plant(
    db: &Database,
    plant: &Plant,
    img_id: Option<usize>,
) -> anyhow::Result<()> {
    let mut conn = db.get_connection().await?;

    r"UPDATE plants 
              SET pollinator_rating = :pollinator_rating,
                  pollinator_reason = :pollinator_reason,
                  bird_rating = :bird_rating,
                  bird_reason = :bird_reason,
                  animal_rating = :animal_rating,
                  animal_reason = :animal_reason,
                  usda_source = :usda_source,
                  wiki_source = :wiki_source,
                  image_id = :image_id
              WHERE id = :id"
        .with(params! {
            "id" => plant.id,
            "pollinator_rating" => plant.pollinator_rating.as_ref().map(|r| r.rating),
            "pollinator_reason" => plant.pollinator_rating.as_ref().map(|r| &r.reason),
            "bird_rating" => plant.bird_rating.as_ref().map(|r| r.rating),
            "bird_reason" => plant.bird_rating.as_ref().map(|r| &r.reason),
            "animal_rating" => plant.animal_rating.as_ref().map(|r| r.rating),
            "animal_reason" => plant.animal_rating.as_ref().map(|r| &r.reason),
            "usda_source" => &plant.usda_source,
            "wiki_source" => &plant.wiki_source,
            "image_id" => img_id
        })
        .ignore(&mut conn)
        .await
        .map_err(|e| anyhow!("update_plant failed to update: {}", e))
}

/// Inserts one plant.  
/// Returns Err if it fails.
pub async fn insert_plant(
    db: &Database,
    plant: &Plant,
    img_id: Option<usize>,
) -> anyhow::Result<usize> {
    let mut conn = db.get_connection().await?;
    println!("inserting plant: {plant:?}");

    r"INSERT INTO plants 
        (scientific_name, common_name, bloom, 
         pollinator_rating, pollinator_reason,
         bird_rating, bird_reason,
         animal_rating, animal_reason,
         usda_source, wiki_source,
         image_id)
      VALUES 
        (:scientific_name, :common_name, :bloom, 
         :pollinator_rating, :pollinator_reason,
         :bird_rating, :bird_reason,
         :animal_rating, :animal_reason,
         :usda_source, :wiki_source,
         :image_id)
            RETURNING id"
        .with(params! {
            "scientific_name" => &plant.scientific,
            "common_name" => &plant.common,
            "bloom" => &plant.bloom,
            "pollinator_rating" => plant.pollinator_rating.as_ref().map(|r| r.rating),
            "pollinator_reason" => plant.pollinator_rating.as_ref().map(|r| r.reason.clone()),
            "bird_rating" => plant.bird_rating.as_ref().map(|r| r.rating),
            "bird_reason" => plant.bird_rating.as_ref().map(|r| r.reason.clone()),
            "animal_rating" => plant.animal_rating.as_ref().map(|r| r.rating),
            "animal_reason" => plant.animal_rating.as_ref().map(|r| r.reason.clone()),
            "usda_source" => &plant.usda_source,
            "wiki_source" => &plant.wiki_source,
            "image_id" => img_id
        })
        .fetch(&mut conn)
        .await
        .map(|ids| ids[0])
        .map_err(|e| anyhow!("save_plant failed to insert: {}", e))
}

/// Selects multiple plants by zip/moisture/shade.  
/// Returns Err if it fails.
pub async fn select_plants_by_zip_moisture_shade(
    db: &Database,
    zip: &str,
    moisture: &Moisture,
    shade: &Shade,
) -> anyhow::Result<Vec<Plant>> {
    let mut conn = db.get_connection().await?;

    r"
SELECT 
  p.id, p.scientific_name, p.common_name, p.bloom, 
  p.pollinator_rating, p.pollinator_reason,
  p.bird_rating, p.bird_reason,
  p.animal_rating, p.animal_reason,
  p.usda_source, p.wiki_source,
  i.id as image_id, i.title, i.card_url, i.original_url, i.author, i.license
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
    .map(&mut conn, |plant: Plant| plant)
    .await
    .map_err(|e| anyhow!(e))
}

/// Selects one plant by scientific name.  
/// Returns Err if it fails, Ok(None) if not found.
pub async fn select_plant_by_scientific_name(
    db: &Database,
    scientific_name: &str,
) -> anyhow::Result<Option<Plant>> {
    let mut conn = db.get_connection().await?;

    r"
SELECT
  p.id, p.scientific_name, p.common_name, p.bloom, 
  p.pollinator_rating, p.pollinator_reason,
  p.bird_rating, p.bird_reason,
  p.animal_rating, p.animal_reason,
  p.usda_source, p.wiki_source,
  i.id as image_id, i.title, i.card_url, i.original_url, i.author, i.license
FROM plants p
INNER JOIN images i ON i.id = p.image_id
WHERE scientific_name = :scientific_name"
        .with(params! {
            "scientific_name" => scientific_name,
        })
        .first(&mut conn)
        .await
        .map_err(|e| anyhow!(e))
}

/// Inserts one image.
/// Returns Err if it fails.
pub async fn insert_image(db: &Database, image: &Image) -> anyhow::Result<usize> {
    let mut conn = db.get_connection().await?;
    r"INSERT INTO images (title, card_url, original_url, author, license)
            VALUES (:title, :card_url, :original_url, :author, :license)
            RETURNING id"
        .with(params! {
            "title" => &image.title,
            "card_url" => &image.card_url,
            "original_url" => &image.original_url,
            "author" => &image.author,
            "license" => &image.license,
        })
        .fetch(&mut conn)
        .await
        .map(|ids| ids[0])
        .map_err(|e| anyhow!("save_image failed to insert: {}", e))
}

/// Selects all nurseries which match the given zipcode.
/// Returns Err if it fails, Ok(empty vec) if none are found.
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
        .map_err(|e| anyhow!(e))
}

/// Selects a region's name for the given zipcode.
/// Returns Err if it fails, Ok(None) if none are found.
pub async fn select_region_name_by_zip(db: &Database, zip: &str) -> anyhow::Result<Option<String>> {
    let mut conn = db.get_connection().await?;

    r"
SELECT name
FROM regions r
INNER JOIN zipcodes z
  ON z.region_id = r.id
WHERE z.zipcode = ?"
        .with((zip,))
        .first(&mut conn)
        .await
        .map_err(|e| anyhow!(e))
}
