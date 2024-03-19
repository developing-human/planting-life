use crate::domain::*;
use anyhow::anyhow;
use mockall::automock;
use mysql_async::{prelude::*, Conn, Opts, Pool};
use std::{collections::HashSet, fmt::Display};
use tracing::log::warn;

// Looks for the closest neighboring zip code to the one provided on both sides,
// then selects the one that is closest.
const SELECT_CLOSEST_ZIP_QUERY: &str = r"
SELECT
  CASE WHEN prev IS NULL THEN nxt
       WHEN nxt IS NULL THEN prev
       WHEN ABS(:zip - IFNULL(prev, nxt)) < ABS(:zip - IFNULL(nxt, prev)) THEN prev
       ELSE nxt
  END AS closest_key
  FROM (
    SELECT :zip as zipcode,
       (SELECT MAX(zipcode) FROM zipcodes AS prev WHERE prev.zipcode < :zip) AS prev,
       (SELECT MIN(zipcode) FROM zipcodes AS nxt WHERE nxt.zipcode > :zip) AS nxt
  ) AS subquery";

pub struct SqlRunner {
    pool: Option<Pool>,
}

#[automock]
impl SqlRunner {
    pub fn new(url: &str) -> Self {
        if Opts::try_from(url).is_err() {
            warn!("Starting server without database!  Caching/nurseries are unavailable.");
            Self { pool: None }
        } else {
            Self {
                pool: Some(Pool::new(url)),
            }
        }
    }

    async fn get_connection(&self) -> anyhow::Result<Conn> {
        if let Some(pool) = &self.pool {
            match pool.get_conn().await {
                Ok(conn) => Ok(conn),
                Err(e) => {
                    warn!("can't get db connection: {}", e);
                    Err(anyhow!("{e}"))
                }
            }
        } else {
            warn!("tried to get db connection, but db is not connected");
            Err(anyhow!("db is not connected"))
        }
    }

    /// Inserts a new Query into the database.
    /// Returns Err if it fails.
    pub async fn upsert_query(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        let queries_result: Result<Option<usize>, mysql_async::Error> =
            r"INSERT INTO queries (moisture, shade, region_id, count) VALUES
            (?, ?, (SELECT region_id from zipcodes where zipcode = ?), 1)
            ON DUPLICATE KEY UPDATE count = count + 1
            "
            .with((moisture.to_string(), shade.to_string(), zip))
            .first(&mut conn)
            .await;

        match queries_result {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("insert into queries failed: {}", e)),
        }
    }

    pub async fn check_zip_exists(&self, zip: &str) -> anyhow::Result<bool> {
        let mut conn = self.get_connection().await?;
        let query_result: Result<Option<u8>, mysql_async::Error> =
            r"SELECT 1 from zipcodes where zipcode = :zip"
                .with(params! {
                    "zip" => zip,
                })
                .first(&mut conn)
                .await;

        match query_result {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(anyhow!("select from zipcodes failed: {e}")),
        }
    }

    pub async fn select_closest_zip(&self, zip: &str) -> anyhow::Result<String> {
        let mut conn = self.get_connection().await?;

        let query_result: Result<Option<usize>, mysql_async::Error> = SELECT_CLOSEST_ZIP_QUERY
            .with(params! {
                "zip" => zip,
            })
            .first(&mut conn)
            .await;

        match query_result {
            // db has this as integer, so format to 5 chars w/ leading zeros
            Ok(Some(closest_zip)) => Ok(format!("{closest_zip:05}")),
            Ok(None) => Err(anyhow!("select_closest_zip closest zip not found")),
            Err(e) => Err(anyhow!("select_closest_zip error finding closest zip: {e}")),
        }
    }

    /// Selects one plant by scientific name.
    /// Returns Err if it fails, Ok(None) if not found.
    pub async fn select_query_count(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> anyhow::Result<usize> {
        let mut conn = self.get_connection().await?;

        r"
SELECT count
FROM queries
WHERE moisture = :moisture
AND shade = :shade
AND region_id = (SELECT region_id from zipcodes where zipcode = :zip)"
            .with(params! {
                "moisture" => moisture.to_string(),
                "shade" => shade.to_string(),
                "zip" => zip,
            })
            .first(&mut conn)
            .await
            .map(|count| count.unwrap_or(0)) // Not found, count as 0
            .map_err(|e| anyhow!("select_query_count failed: {e}"))
    }

    /// Inserts into regions_plants.
    /// Returns Err if it fails.
    pub async fn insert_region_plants(
        &self,
        zip: &str,
        plant_ids: HashSet<usize>,
    ) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;

        // Some rows could already exist, this ignores duplicate key errors
        // The "dummy update" is required to make this statement valid.
        r"INSERT INTO regions_plants (region_id, plant_id)
            VALUES ((SELECT region_id from zipcodes where zipcode = :zip), :plant_id)
            ON DUPLICATE KEY UPDATE region_id=region_id, plant_id=plant_id"
            .with(plant_ids.iter().map(|id| {
                params! {
                    "zip" => zip,
                    "plant_id" => id
                }
            }))
            .batch(&mut conn)
            .await
            .map_err(|e| anyhow!(e))
    }

    /// Updates one plant.
    /// Returns Err if it fails.
    pub async fn update_plant(&self, plant: &Plant, img_id: Option<usize>) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;

        r"UPDATE plants
              SET pollinator_rating = :pollinator_rating,
                  bird_rating = :bird_rating,

                  spread_rating = :spread_rating,
                  deer_resistance_rating = :deer_resistance_rating,

                  usda_source = :usda_source,
                  wiki_source = :wiki_source,

                  bloom = :bloom,
                  height = :height,
                  spread = :spread,

                  moistures = :moistures,
                  shades = :shades,

                  image_id = :image_id
              WHERE id = :id"
            .with(params! {
                "id" => plant.id,

                "pollinator_rating" => plant.pollinator_rating,
                "bird_rating" => plant.bird_rating,

                "spread_rating" => plant.spread_rating,
                "deer_resistance_rating" => plant.deer_resistance_rating,

                "usda_source" => &plant.usda_source,
                "wiki_source" => &plant.wiki_source,

                "bloom" => &plant.bloom,
                "height" => &plant.height,
                "spread" => &plant.spread,

                "moistures" => to_comma_separated_string(&plant.moistures),
                "shades" => to_comma_separated_string(&plant.shades),

                "image_id" => img_id
            })
            .ignore(&mut conn)
            .await
            .map_err(|e| anyhow!("update_plant failed to update: {}", e))
    }

    /// Inserts one plant.
    /// Returns Err if it fails.
    pub async fn insert_plant(
        &self,
        plant: &Plant,
        img_id: Option<usize>,
    ) -> anyhow::Result<usize> {
        let mut conn = self.get_connection().await?;

        r"INSERT INTO plants
        (scientific_name, common_name,
         bloom, height, spread,
         moistures, shades,
         pollinator_rating,
         bird_rating,
         spread_rating, deer_resistance_rating,
         usda_source, wiki_source,
         image_id)
      VALUES
        (:scientific_name, :common_name,
         :bloom, :height, :spread,
         :moistures, :shades,
         :pollinator_rating,
         :bird_rating,
         :spread_rating, :deer_resistance_rating,
         :usda_source, :wiki_source,
         :image_id)
            RETURNING id"
            .with(params! {
                "scientific_name" => &plant.scientific,
                "common_name" => &plant.common,

                "bloom" => &plant.bloom,
                "height" => &plant.height,
                "spread" => &plant.spread,

                "pollinator_rating" => plant.pollinator_rating,
                "bird_rating" => plant.bird_rating,

                "spread_rating" => plant.spread_rating,
                "deer_resistance_rating" => plant.deer_resistance_rating,

                "usda_source" => &plant.usda_source,
                "wiki_source" => &plant.wiki_source,

                "moistures" => to_comma_separated_string(&plant.moistures),
                "shades" => to_comma_separated_string(&plant.shades),

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
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> anyhow::Result<Vec<Plant>> {
        let mut conn = self.get_connection().await?;

        r"
SELECT
  p.id, p.scientific_name, p.common_name,
  p.bloom, p.height, p.spread,
  p.moistures, p.shades,
  p.pollinator_rating,
  p.bird_rating,
  p.spread_rating, p.deer_resistance_rating,
  p.usda_source, p.wiki_source,
  i.id as image_id, i.title, i.card_url, i.original_url, i.author, i.license
FROM plants p

INNER JOIN regions_plants rp on rp.plant_id = p.id
INNER JOIN zipcodes z ON z.region_id = rp.region_id
LEFT JOIN images i ON i.id = p.image_id
WHERE z.zipcode = :zipcode
  AND (p.moistures is NULL OR FIND_IN_SET(:moisture, p.moistures))
  AND (p.shades is NULL OR FIND_IN_SET(:shade, p.shades))
ORDER BY
  p.moistures IS NOT NULL and p.shades IS NOT NULL desc,
  POW(p.pollinator_rating, 3) + POW(p.bird_rating, 3) desc

"
        .with(params! {
            "zipcode" => zip,
            "moisture" => moisture.to_string(),
            "shade" => shade.to_string(),
        })
        .map(&mut conn, |plant: Plant| plant)
        .await
        .map_err(|e| anyhow!(e))
    }

    /// Selects one plant by scientific name.
    /// Returns Err if it fails, Ok(None) if not found.
    pub async fn select_plant_by_scientific_name(
        &self,
        scientific_name: &str,
    ) -> anyhow::Result<Option<Plant>> {
        let mut conn = self.get_connection().await?;

        r"
SELECT
  p.id, p.scientific_name, p.common_name,
  p.bloom, p.height, p.spread,
  p.moistures, p.shades,
  p.pollinator_rating,
  p.bird_rating,
  p.spread_rating, p.deer_resistance_rating,
  p.usda_source, p.wiki_source,
  i.id as image_id, i.title, i.card_url, i.original_url, i.author, i.license
FROM plants p
LEFT JOIN images i ON i.id = p.image_id
WHERE scientific_name = :scientific_name"
            .with(params! {
                "scientific_name" => scientific_name,
            })
            .first(&mut conn)
            .await
            .map_err(|e| anyhow!(e))
    }

    /// Selects one plant by id.
    /// Returns Err if it fails, Ok(None) if not found.
    pub async fn select_plant_by_id(&self, id: usize) -> anyhow::Result<Option<Plant>> {
        let mut conn = self.get_connection().await?;

        r"
SELECT
  p.id, p.scientific_name, p.common_name,
  p.bloom, p.height, p.spread,
  p.moistures, p.shades,
  p.pollinator_rating,
  p.bird_rating,
  p.spread_rating, p.deer_resistance_rating,
  p.usda_source, p.wiki_source,
  i.id as image_id, i.title, i.card_url, i.original_url, i.author, i.license
FROM plants p
LEFT JOIN images i ON i.id = p.image_id
WHERE p.id = :id"
            .with(params! {
                "id" => id,
            })
            .first(&mut conn)
            .await
            .map_err(|e| anyhow!(e))
    }

    /// Inserts one image.
    /// Returns Err if it fails.
    pub async fn insert_image(&self, image: &Image) -> anyhow::Result<usize> {
        let mut conn = self.get_connection().await?;
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
    pub async fn select_nurseries_by_zip(&self, zip: &str) -> anyhow::Result<Vec<Nursery>> {
        let mut conn = self.get_connection().await?;

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
    pub async fn select_region_name_by_zip(&self, zip: &str) -> anyhow::Result<Option<String>> {
        let mut conn = self.get_connection().await?;

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

    pub async fn select_garden_by_id(
        &self,
        id: &str,
        read_only: bool,
    ) -> anyhow::Result<Option<Garden>> {
        let mut conn = self.get_connection().await?;

        let id_field_name = if read_only { "read_id" } else { "write_id" };

        format!(
            "
SELECT g.name, g.zipcode, r.name, shade, moisture, g.read_id
FROM gardens g
INNER JOIN zipcodes z ON z.zipcode = g.zipcode
INNER JOIN regions r ON r.id = z.region_id
WHERE {id_field_name} = ?"
        )
        .with((id,))
        .first(&mut conn)
        .await
        .map_err(|e| anyhow!(e))
    }

    pub async fn select_plants_by_garden_id(
        &self,
        garden_id: &str,
        read_only: bool,
    ) -> anyhow::Result<Vec<Plant>> {
        let mut conn = self.get_connection().await?;

        let id_field_name = if read_only { "read_id" } else { "write_id" };

        format!(
            r"
SELECT
  p.id, p.scientific_name, p.common_name,
  p.bloom, p.height, p.spread,
  p.moistures, p.shades,
  p.pollinator_rating,
  p.bird_rating,
  p.spread_rating, p.deer_resistance_rating,
  p.usda_source, p.wiki_source,
  i.id as image_id, i.title, i.card_url, i.original_url, i.author, i.license
FROM plants p
INNER JOIN gardens_plants gp on gp.plant_id = p.id
INNER JOIN gardens g on g.id = gp.garden_id
LEFT JOIN images i ON i.id = p.image_id
WHERE g.{id_field_name} = :garden_id
ORDER BY gp.ordering
"
        )
        .with(params! {
            "garden_id" => garden_id,
        })
        .map(&mut conn, |plant: Plant| plant)
        .await
        .map_err(|e| anyhow!(e))
    }

    /// Inserts a Garden (but not the plants!), returning its id.
    pub async fn insert_garden(
        &self,
        garden: &Garden,
        read_id: &str,
        write_id: &str,
    ) -> anyhow::Result<usize> {
        let mut conn = self.get_connection().await?;
        r"INSERT INTO gardens (read_id, write_id, name, shade, moisture, zipcode)
            VALUES (:read_id, :write_id, :name, :shade, :moisture, :zipcode)
            RETURNING id"
            .with(params! {
                "read_id" => read_id,
                "write_id" => write_id,
                "name" => &garden.name,
                "shade" => garden.shade.to_string(),
                "moisture" => garden.moisture.to_string(),
                "zipcode" => &garden.zipcode,
            })
            .fetch(&mut conn)
            .await
            .map(|ids| ids[0])
            .map_err(|e| anyhow!("insert_garden failed: {}", e))
    }

    /// Updates an existing Garden (but not the plants!).
    pub async fn update_garden(&self, write_id: &str, name: &str) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;

        r"UPDATE gardens
              SET name = :name
              WHERE write_id = :write_id"
            .with(params! {
                "write_id" => write_id,

                "name" => name

            })
            .ignore(&mut conn)
            .await
            .map_err(|e| anyhow!("update_garden failed: {}", e))
    }

    pub async fn replace_garden_plants(
        &self,
        write_id: &str,
        plant_ids: Vec<usize>,
    ) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        let mut transaction = conn
            .start_transaction(mysql_async::TxOpts::default())
            .await?;

        "DELETE gp FROM gardens_plants gp
            INNER JOIN gardens g on g.id = gp.garden_id
            WHERE write_id = :write_id"
            .with(params! {
                "write_id" => write_id
            })
            .ignore(&mut transaction)
            .await
            .map_err(|e| anyhow!("replace_garden_plants delete failed: {e}"))?;

        "INSERT INTO gardens_plants (garden_id, plant_id, ordering)
           VALUES ((SELECT id from gardens where write_id = :write_id), :plant_id, :ordering)"
            .with(plant_ids.iter().enumerate().map(|(ordering, id)| {
                params! {
                    "write_id" => write_id,
                    "plant_id" => id,
                    "ordering" => ordering
                }
            }))
            .batch(&mut transaction)
            .await
            .map_err(|e| anyhow!("replace_garden_plants insert failed: {e}"))?;

        transaction
            .commit()
            .await
            .map_err(|e| anyhow!("replace_garden_plants commit failed: {e}"))
    }

    pub async fn find_plants_by_word_prefix(&self, expression: &str) -> anyhow::Result<Vec<Plant>> {
        let mut conn = self.get_connection().await?;

        r"
 SELECT id, scientific_name, common_name 
 FROM plants 
 WHERE MATCH(scientific_name, common_name) AGAINST (:expression IN BOOLEAN MODE)
 LIMIT 10
"
        .with(params! {
            "expression" => expression
        })
        .map(&mut conn, |plant: Plant| plant)
        .await
        .map_err(|e| anyhow!(e))
    }

    /*
    /// Checks if a read_id or write_id already exists.
    /// Note: Currently untested/unused.
    pub async fn _check_garden_id_exists(&self, id: &str, field: &str) -> anyhow::Result<bool> {
        let mut conn = self.get_connection().await?;
        let query_result: Result<Option<u8>, mysql_async::Error> =
            format!("SELECT 1 from gardens where {field} = :id")
                .with(params! {
                    "id" => id,
                })
                .first(&mut conn)
                .await;

        match query_result {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(anyhow!("select from gardens failed: {e}")),
        }
    }
    */
}

fn to_comma_separated_string<T: Display>(vec: &[T]) -> Option<String> {
    // If the vector is empty, we want to keep these as null in the db
    // A null value indicates we should try to populate it again next time
    if vec.is_empty() {
        return None;
    }

    Some(
        vec.iter()
            .map(|m| m.to_string())
            .collect::<Vec<String>>()
            .join(","),
    )
}
