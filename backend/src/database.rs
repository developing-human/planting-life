use std::collections::HashSet;

use crate::domain::*;
use anyhow::anyhow;
use async_trait::async_trait;
use futures::future;
use mysql_async::Conn;
use mysql_async::Opts;
use mysql_async::Pool;
use tracing::log::warn;

mod conversions;
mod sql;

#[async_trait]
pub trait Database: Send + Sync {
    /// Finds all Nurseries near the given zipcode.
    async fn find_nurseries(&self, zip: &str) -> Vec<Nursery>;

    /// Finds the closest valid zipcode, returns Err if it can't.
    async fn lookup_closest_valid_zip(&self, zip: &str) -> anyhow::Result<String>;

    /// Finds all Plants which match the given parameters.
    async fn lookup_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> Vec<Plant>;

    ///Saves a new Query and maps it to the plants referenced by plant_ids.
    ///
    ///Failures are logged, but are otherwise ignored.
    async fn save_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
        all_plants: Vec<Plant>,
        saved_plants: Vec<Plant>,
    );

    async fn save_plant_region(&self, plant: &Plant, zip: &str);

    /// Returns how many times the query for these parameters has been executed
    ///
    /// Failures are logged, but are otherwise ignored.
    async fn get_query_count(&self, zip: &str, moisture: &Moisture, shade: &Shade) -> usize;

    /// Saves a list of Plants, returning a list of new Plants which
    /// have their ids populated.  Returns Err if any fail to save.
    async fn save_plants(&self, plants_in: &Vec<&Plant>) -> anyhow::Result<Vec<Plant>>;

    /// Inserts or updates a single Plant, returning a new Plant with its
    /// id populated. Returns Err if it fails to save.
    async fn save_plant(&self, plant: &Plant) -> anyhow::Result<Plant>;

    /// Saves an Image, returning a new Image with the database id populated.
    /// Returns Err if it fails to save.
    async fn save_image(&self, image: &Image) -> anyhow::Result<Image>;

    /// Fetches one Plant by scientific name.  Returns None if it is not
    /// found or if there is a database error.
    async fn get_plant_by_scientific_name(&self, scientific_name: &str) -> Option<Plant>;

    /// Fetches the region name for a zipcodes.
    /// Returns None if not found or if there is a database error.
    async fn get_region_name_by_zip(&self, zip: &str) -> Option<String>;
}

#[derive(Clone)]
pub struct MariaDB {
    pool: Option<Pool>,
}

impl MariaDB {
    /// Creates a Database.  If the url is None, it creates one without a pool.
    /// When the pool is None, it degrades gracefully.
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
}

#[async_trait]
impl Database for MariaDB {
    async fn find_nurseries(&self, zip: &str) -> Vec<Nursery> {
        sql::select_nurseries_by_zip(self, zip)
            .await
            .unwrap_or_else(|e| {
                warn!("find_nurseries query failed: {}", e);
                vec![]
            })
    }

    async fn lookup_closest_valid_zip(&self, zip: &str) -> anyhow::Result<String> {
        if zip.len() != 5 || !zip.chars().all(char::is_numeric) {
            return Err(anyhow!("invalid zipcode format: {zip}"));
        }

        if sql::check_zip_exists(self, zip).await? {
            Ok(zip.to_string())
        } else {
            Ok(sql::select_closest_zip(self, zip).await?)
        }
    }

    async fn lookup_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> Vec<Plant> {
        sql::select_plants_by_zip_moisture_shade(self, zip, moisture, shade)
            .await
            .unwrap_or_else(|e| {
                warn!("lookup_query_results query failed: {}", e);
                vec![]
            })
    }

    async fn save_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
        all_plants: Vec<Plant>,
        saved_plants: Vec<Plant>,
    ) {
        if let Err(e) = sql::upsert_query(self, zip, moisture, shade).await {
            // Log this failure, but continue on
            warn!("save_query_results failed to upsert query: {e}")
        }

        // Some plants in plants_with_images may be missing ids.  Merging with
        // saved_plants will find all of them.
        let plant_ids: HashSet<usize> = all_plants
            .iter()
            .chain(saved_plants.iter())
            .filter_map(|p| p.id)
            .collect();

        if let Err(e) = sql::insert_region_plants(self, zip, plant_ids).await {
            warn!("save_query_results failed to insert region plants: {}", e);
        }
    }

    async fn save_plant_region(&self, plant: &Plant, zip: &str) {
        if plant.id.is_none() {
            warn!("save_plant_region requires plant.id");
            return;
        }

        // I don't love this, maybe make a better interface to insert_region_plants.
        let mut plant_ids = HashSet::new();
        plant_ids.insert(plant.id.unwrap());

        if let Err(e) = sql::insert_region_plants(self, zip, plant_ids).await {
            warn!("save_query_results failed to insert region plants: {}", e);
        }
    }

    async fn get_query_count(&self, zip: &str, moisture: &Moisture, shade: &Shade) -> usize {
        match sql::select_query_count(self, zip, moisture, shade).await {
            Ok(count) => count,
            Err(e) => {
                warn!("get_query_count failed to select count, returning zero: {e}");
                0
            }
        }
    }

    async fn save_plants(&self, plants_in: &Vec<&Plant>) -> anyhow::Result<Vec<Plant>> {
        let mut futures = vec![];
        for plant in plants_in {
            futures.push(self.save_plant(plant));
        }

        // collect() here is practically magic,
        // converting Vec<Result<Plant>> into Result<Vec<Plant>>
        future::join_all(futures).await.into_iter().collect()
    }

    async fn save_plant(&self, plant: &Plant) -> anyhow::Result<Plant> {
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
            sql::update_plant(self, plant, img_id).await?;
            id
        } else {
            sql::insert_plant(self, plant, img_id).await?
        };

        Ok(Plant {
            id: Some(id),
            ..plant.clone()
        })
    }

    async fn save_image(&self, image: &Image) -> anyhow::Result<Image> {
        let id = sql::insert_image(self, image).await;

        id.map(|id| Image {
            id: Some(id),
            ..image.clone()
        })
    }

    async fn get_plant_by_scientific_name(&self, scientific_name: &str) -> Option<Plant> {
        match sql::select_plant_by_scientific_name(self, scientific_name).await {
            Ok(Some(plant)) => Some(plant),
            Ok(None) => None,
            Err(e) => {
                warn!("get_plant_by_scientific_name failed to select: {e}");
                None
            }
        }
    }

    async fn get_region_name_by_zip(&self, zip: &str) -> Option<String> {
        match sql::select_region_name_by_zip(self, zip).await {
            Ok(Some(name)) => Some(name),
            Ok(None) => {
                warn!("get_region_name_by_zip could not find region name");
                None
            }
            Err(e) => {
                warn!("get_region_name_by_zip failed to select: {e}");
                None
            }
        }
    }
}
