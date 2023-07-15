use std::collections::HashSet;

use crate::domain::*;
use anyhow::anyhow;
use futures::future;
use mysql_async::Conn;
use mysql_async::Opts;
use mysql_async::Pool;
use tracing::log::warn;

mod sql;

#[derive(Clone)]
pub struct Database {
    pool: Option<Pool>,
}

impl Database {
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

    /// Finds all Nurseries near the given zipcode.
    pub async fn find_nurseries(&self, zip: &str) -> Vec<Nursery> {
        sql::select_nurseries_by_zip(self, zip)
            .await
            .unwrap_or_else(|e| {
                warn!("find_nurseries query failed: {}", e);
                vec![]
            })
    }

    /// Finds all NativePlants which match the given parameters.
    pub async fn lookup_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> Vec<NativePlant> {
        sql::select_plants_by_zip_moisture_shade(self, zip, moisture, shade)
            .await
            .unwrap_or_else(|e| {
                warn!("lookup_query_results query failed: {}", e);
                vec![]
            })
    }

    ///Saves a new Query and maps it to the plants referenced by plant_ids.
    ///
    ///Failures are logged, but are otherwise ignored.
    pub async fn save_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
        all_plants: Vec<NativePlant>,
        saved_plants: Vec<NativePlant>,
    ) {
        // Also, don't save the results of this query if we have fewer than the desired number,
        // this should be a rare occurance and this is a simple way to handle it.  The
        // alternative would be to (on fetch) get some from the database and the rest from gpt.
        // Its easier to just get all from gpt, even if its a little more work.
        if all_plants.len() != 12 {
            warn!("only have {} plants, not caching", all_plants.len());
            return;
        }

        // At least one plant is missing an image, so don't store these results.  Very
        // occasionally we'll run into this, and its okay as a quirk but lets not cache
        // it forever.
        let plant_without_image = all_plants.iter().find(|p| p.image.is_none());
        if let Some(plant_without_image) = plant_without_image {
            warn!(
                "not all plants have an image (missing for {}), not caching",
                plant_without_image.scientific
            );
            return;
        }

        // Some plants in all_plants may be missing ids.  Merging with saved_plants will
        // find all of them.
        let plant_ids: HashSet<usize> = all_plants
            .iter()
            .chain(saved_plants.iter())
            .filter_map(|p| p.id)
            .collect();

        let query_id = match sql::insert_query(self, zip, moisture, shade).await {
            Ok(id) => id,
            Err(e) => {
                warn!("save_query_results failed to insert Query: {}", e);
                return;
            }
        };

        if let Err(e) = sql::insert_query_plants(self, query_id, plant_ids).await {
            warn!("save_query_results failed to insert Query Plants: {}", e);
        }
    }

    /// Saves a list of NativePlants, returning a list of new NativePlants which
    /// have their ids populated.  Returns Err if any fail to save.
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

    /// Inserts or updates a single NativePlant, returning a new NativePlant with its
    /// id populated. Returns Err if it fails to save.
    pub async fn save_plant(&self, plant: &NativePlant) -> anyhow::Result<NativePlant> {
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

        Ok(NativePlant {
            id: Some(id),
            ..plant.clone()
        })
    }

    /// Saves an Image, returning a new Image with the database id populated.
    /// Returns Err if it fails to save.
    async fn save_image(&self, image: &Image) -> anyhow::Result<Image> {
        let id = sql::insert_image(self, image).await;

        id.map(|id| Image {
            id: Some(id),
            ..image.clone()
        })
    }

    /// Fetches one NativePlant by scientific name.  Returns None if it is not
    /// found or if there is a database error.
    pub async fn get_plant_by_scientific_name(&self, scientific_name: &str) -> Option<NativePlant> {
        match sql::select_plant_by_scientific_name(self, scientific_name).await {
            Ok(Some(plant)) => Some(plant),
            Ok(None) => None,
            Err(e) => {
                warn!("get_plant_by_scientific_name failed to select: {e}");
                None
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
