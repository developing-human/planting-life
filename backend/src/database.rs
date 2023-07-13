use std::collections::HashSet;

use crate::domain::*;
use anyhow::anyhow;
use futures::future;
use mysql_async::prelude::*;
use mysql_async::Conn;
use mysql_async::Opts;
use mysql_async::Pool;
use tracing::log::warn;

#[derive(Clone)]
pub struct Database {
    pool: Option<Pool>,
}

impl Database {
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

    pub async fn find_nurseries(&self, zip: &str) -> Vec<Nursery> {
        let mut conn = match self.get_connection().await {
            Ok(conn) => conn,
            Err(_) => return vec![],
        };

        r"
SELECT miles, name, url, address, city, state, n.zipcode
FROM zipcodes_nurseries zn
INNER JOIN nurseries n 
  ON n.id = zn.nursery_id 
WHERE zn.zipcode = ?
ORDER BY miles ASC"
            .with((zip,))
            .map(
                &mut conn,
                |(miles, name, url, address, city, state, zip)| Nursery {
                    name,
                    url,
                    address,
                    city,
                    state,
                    zip,
                    miles,
                    map_url: None,
                },
            )
            .await
            .unwrap_or_else(|e| {
                warn!("find_nurseries query failed: {}", e);
                vec![]
            })
    }

    pub async fn lookup_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> Vec<NativePlant> {
        let mut conn = match self.get_connection().await {
            Ok(conn) => conn,
            Err(_) => return vec![],
        };

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
        .map(&mut conn, |(id, scientific, common, bloom, description, img_id, title, card_url, original_url, author, license)| {

            // Everything related to the image is optional because the image may not exist
            // But if img_id is present, everything else is required.  Hence the unwraps.
            let img_id: Option<usize> = img_id;
            let card_url: Option<String> = card_url;
            let original_url: Option<String> = original_url;
            let author: Option<String> = author;
            let license: Option<String> = license;
            let title: Option<String> = title;

            let scientific: String = scientific;
            let bloom: Option<String> = bloom;
            NativePlant {
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
                        title: title.unwrap(),
                        card_url: card_url.unwrap(),
                        original_url: original_url.unwrap(),
                        author: author.unwrap(),
                        license_url: Image::get_license_url(&license).unwrap(),
                        license,
                    }
                })
            }
        })
        .await
        .unwrap_or_else(|e| {
            warn!("lookup_query_results query failed: {}", e);
            vec![]
        })
    }

    /* Saves a new Query and maps it to the plants referenced by plant_ids.
     *
     * Failures are logged, but are otherwise ignored.
     */
    pub async fn save_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
        plant_ids: HashSet<usize>,
    ) {
        let mut conn = match self.get_connection().await {
            Ok(conn) => conn,
            Err(_) => return,
        };

        let queries_result: Result<Option<i32>, mysql_async::Error> =
            r"INSERT INTO queries (moisture, shade, region_id) VALUES
            (?, ?, (SELECT region_id from zipcodes where zipcode = ?))
            RETURNING id"
                .with((moisture.to_string(), shade.to_string(), zip))
                .first(&mut conn)
                .await;

        let query_id = match queries_result {
            Ok(Some(id)) => id,
            Ok(None) => {
                warn!("save_query_results saved query but did not receive id");
                return;
            }
            Err(e) => {
                warn!("save_query_results insert into queries failed: {}", e);
                return;
            }
        };

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
            .unwrap_or_else(|e| {
                warn!("save_query_results query failed: {}", e);
            });
    }

    // Takes in a vector of plants which are not in the database (null ids), and
    // returns a new vector of native plants which have ids and are in the database
    pub async fn save_plants(
        &self,
        plants_in: &Vec<&NativePlant>,
    ) -> anyhow::Result<Vec<NativePlant>> {
        let mut futures = vec![];
        for plant in plants_in {
            futures.push(self.save_plant(plant));
        }

        // collect() here is practically magic,
        // converting Vec<Result<NativePlant>> into Result<Vec<NativePlant>>
        future::join_all(futures).await.into_iter().collect()
    }

    /* Takes in a plant which may or may not be in the database, and returns
     * a new plant with its database id populated.
     */
    pub async fn save_plant(&self, plant: &NativePlant) -> anyhow::Result<NativePlant> {
        let mut conn = match self.get_connection().await {
            Ok(conn) => conn,
            Err(_) => return Err(anyhow!("can't get db connection")),
        };

        let mut img_id = None;
        if let Some(image) = &plant.image {
            img_id = image.id;
            if image.id.is_none() {
                if let Ok(saved_image) = self.save_image(image).await {
                    img_id = saved_image.id;
                }
            }
        }

        let id = if let Some(id) = plant.id {
            r"UPDATE plants 
              SET description = :description, image_id = :image_id
              WHERE id = :id"
                .with(params! {
                    "id" => id,
                    "description" => plant.description.clone(),
                    "image_id" => img_id
                })
                .ignore(&mut conn)
                .await
                .map(|_| id)
                .map_err(|e| anyhow!("save_plant failed to update: {}", e))
        } else {
            r"INSERT INTO plants (scientific_name, common_name, bloom, description, image_id)
            VALUES (:scientific_name, :common_name, :bloom, :description, :image_id)
            RETURNING id"
                .with(params! {
                    "scientific_name" => &plant.scientific,
                    "common_name" => &plant.common,
                    "bloom" => &plant.bloom,
                    "description" => plant.description.clone().unwrap_or("null".to_string()),
                    "image_id" => img_id
                })
                .fetch(&mut conn)
                .await
                .map(|ids| ids[0])
                .map_err(|e| anyhow!("save_plant failed to insert: {}", e))
        };

        id.map(|id| NativePlant {
            id: Some(id),
            scientific: plant.scientific.clone(),
            common: plant.common.clone(),
            description: plant.description.clone(),
            bloom: None,
            image: None,
        })
    }

    // Saves the given image, returning a new Image with the database id populated.
    async fn save_image(&self, image: &Image) -> anyhow::Result<Image> {
        let mut conn = match self.get_connection().await {
            Ok(conn) => conn,
            Err(_) => return Err(anyhow!("can't get db connection")),
        };

        let id = r"INSERT INTO images (title, card_url, original_url, author, license)
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
            .map_err(|e| anyhow!("save_image failed to insert: {}", e));

        id.map(|id| Image {
            id: Some(id),
            title: image.title.clone(),
            card_url: image.card_url.clone(),
            original_url: image.original_url.clone(),
            author: image.author.clone(),
            license: image.license.clone(),
            scientific_name: image.scientific_name.clone(),
            license_url: image.license_url.clone(),
        })
    }

    pub async fn get_plant_by_scientific_name(&self, scientific_name: &str) -> Option<NativePlant> {
        let mut conn = match self.get_connection().await {
            Ok(conn) => conn,
            Err(_) => return None,
        };

        let query_result = r"
SELECT p.id, p.common_name, p.bloom, p.description, i.id, i.title, i.card_url, i.original_url, i.author, i.license
FROM plants p
INNER JOIN images i ON i.id = p.image_id
WHERE scientific_name = :scientific_name"
            .with(params! {
                "scientific_name" => scientific_name,
            })
            .first(&mut conn)
            .await;

        // warns on error, returns None if None (due to ? at end)
        let query_result = match query_result {
            Ok(qr) => qr,
            Err(e) => {
                warn!("get_plant_by_scientific_name query failed: {e}");
                return None;
            }
        }?;

        let (
            id,
            common,
            bloom,
            description,
            img_id,
            title,
            card_url,
            original_url,
            author,
            license,
        ) = query_result;

        // Type information is needed below, and other attempts weren't working :)
        let img_id: Option<usize> = img_id;
        let license: String = license;

        Some(NativePlant {
            id: Some(id),
            scientific: scientific_name.to_string(),
            common,
            description,
            bloom,
            image: img_id.map(|_| Image {
                id: img_id,
                scientific_name: scientific_name.to_string(),
                title,
                card_url,
                original_url,
                author,
                license_url: Image::get_license_url(&license).unwrap(),
                license,
            }),
        })
    }

    async fn get_connection(&self) -> Result<Conn, ()> {
        if let Some(pool) = &self.pool {
            match pool.get_conn().await {
                Ok(conn) => Ok(conn),
                Err(e) => {
                    warn!("can't get db connection: {}", e);
                    Err(())
                }
            }
        } else {
            warn!("tried to get db connection, but db is not connected");
            Err(())
        }
    }
}
