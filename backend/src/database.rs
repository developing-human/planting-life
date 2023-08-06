use crate::domain::*;
use anyhow::anyhow;
use mockall::automock;
use mockall_double::double;
use std::collections::HashSet;
use tracing::log::warn;

#[double]
use self::sql::SqlRunner;

mod conversions;
pub mod sql;

pub struct Database {
    sql_runner: SqlRunner,
}

#[automock]
impl Database {
    /// Creates a Database.  If the url is None, it creates one without a pool.
    /// When the pool is None, it degrades gracefully.
    pub fn new(db_url: &str) -> Self {
        Self {
            sql_runner: SqlRunner::new(db_url),
        }
    }

    /// Finds all Nurseries near the given zipcode.
    pub async fn find_nurseries(&self, zip: &str) -> Vec<Nursery> {
        self.sql_runner
            .select_nurseries_by_zip(zip)
            .await
            .unwrap_or_else(|e| {
                warn!("find_nurseries query failed: {}", e);
                vec![]
            })
    }

    /// Finds the closest valid zipcode, returns Err if it can't.
    pub async fn lookup_closest_valid_zip(&self, zip: &str) -> anyhow::Result<String> {
        if zip.len() != 5 || !zip.chars().all(char::is_numeric) {
            return Err(anyhow!("invalid zipcode format: {zip}"));
        }

        if self.sql_runner.check_zip_exists(zip).await? {
            Ok(zip.to_string())
        } else {
            Ok(self.sql_runner.select_closest_zip(zip).await?)
        }
    }

    /// Finds all Plants which match the given parameters.
    pub async fn lookup_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> Vec<Plant> {
        self.sql_runner
            .select_plants_by_zip_moisture_shade(zip, moisture, shade)
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
        all_plants: Vec<Plant>,
        saved_plants: Vec<Plant>,
    ) {
        if let Err(e) = self.sql_runner.upsert_query(zip, moisture, shade).await {
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

        if let Err(e) = self.sql_runner.insert_region_plants(zip, plant_ids).await {
            warn!("save_query_results failed to insert region plants: {}", e);
        }
    }

    pub async fn save_plant_region(&self, plant: &Plant, zip: &str) {
        if plant.id.is_none() {
            warn!("save_plant_region requires plant.id");
            return;
        }

        // I don't love this, maybe make a better interface to insert_region_plants.
        let mut plant_ids = HashSet::new();
        plant_ids.insert(plant.id.unwrap());

        if let Err(e) = self.sql_runner.insert_region_plants(zip, plant_ids).await {
            warn!("save_query_results failed to insert region plants: {}", e);
        }
    }

    /// Returns how many times the query for these parameters has been executed
    ///
    /// Failures are logged, but are otherwise ignored.
    pub async fn get_query_count(&self, zip: &str, moisture: &Moisture, shade: &Shade) -> usize {
        match self
            .sql_runner
            .select_query_count(zip, moisture, shade)
            .await
        {
            Ok(count) => count,
            Err(e) => {
                warn!("get_query_count failed to select count, returning zero: {e}");
                0
            }
        }
    }

    /// Inserts or updates a single Plant, returning a new Plant with its
    /// id populated. Returns Err if it fails to save.
    pub async fn save_plant(&self, plant: &Plant) -> anyhow::Result<Plant> {
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
            self.sql_runner.update_plant(plant, img_id).await?;
            id
        } else {
            self.sql_runner.insert_plant(plant, img_id).await?
        };

        Ok(Plant {
            id: Some(id),
            ..plant.clone()
        })
    }

    /// Saves an Image, returning a new Image with the database id populated.
    /// Returns Err if it fails to save.
    pub async fn save_image(&self, image: &Image) -> anyhow::Result<Image> {
        let id = self.sql_runner.insert_image(image).await;

        id.map(|id| Image {
            id: Some(id),
            ..image.clone()
        })
    }

    /// Fetches one Plant by scientific name.  Returns None if it is not
    /// found or if there is a database error.
    pub async fn get_plant_by_scientific_name(&self, scientific_name: &str) -> Option<Plant> {
        match self
            .sql_runner
            .select_plant_by_scientific_name(scientific_name)
            .await
        {
            Ok(Some(plant)) => Some(plant),
            Ok(None) => None,
            Err(e) => {
                warn!("get_plant_by_scientific_name failed to select: {e}");
                None
            }
        }
    }

    /// Fetches the region name for a zipcodes.
    /// Returns None if not found or if there is a database error.
    pub async fn get_region_name_by_zip(&self, zip: &str) -> Option<String> {
        match self.sql_runner.select_region_name_by_zip(zip).await {
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

#[cfg(test)]
mod tests {
    use super::{sql::MockSqlRunner, *};

    #[tokio::test]
    async fn test_lookup_closest_valid_zip_too_short() {
        let db = make_db();

        let result = db.lookup_closest_valid_zip("4308").await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid zipcode format: 4308"
        );
    }

    #[tokio::test]
    async fn test_lookup_closest_valid_zip_non_numeric() {
        let db = make_db();

        let result = db.lookup_closest_valid_zip("4308z").await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid zipcode format: 4308z"
        );
    }

    #[tokio::test]
    async fn test_lookup_closest_valid_zip_exists() {
        let db = make_db_with_mock(|mock| {
            mock.expect_check_zip_exists().returning(|_| Ok(true));
        });

        let result = db.lookup_closest_valid_zip("43083").await;

        assert_eq!(result.unwrap(), "43083");
    }

    #[tokio::test]
    async fn test_lookup_closest_valid_zip_exists_err() {
        let db = make_db_with_mock(|mock| {
            mock.expect_check_zip_exists()
                .returning(|_| Err(anyhow!("oops")));
        });

        let result = db.lookup_closest_valid_zip("43083").await;
        assert_eq!(result.unwrap_err().to_string(), "oops")
    }

    #[tokio::test]
    async fn test_lookup_closest_valid_zip_not_exists() {
        let db = make_db_with_mock(|mock| {
            mock.expect_check_zip_exists().returning(|_| Ok(false));
            mock.expect_select_closest_zip()
                .returning(|_| Ok("43081".into()));
        });

        let result = db.lookup_closest_valid_zip("43083").await;
        assert_eq!(result.unwrap(), "43081")
    }

    #[tokio::test]
    async fn test_lookup_closest_valid_zip_not_exists_err() {
        let db = make_db_with_mock(|mock| {
            mock.expect_check_zip_exists().returning(|_| Ok(false));
            mock.expect_select_closest_zip()
                .returning(|_| Err(anyhow!("oops")));
        });

        let result = db.lookup_closest_valid_zip("43083").await;
        assert_eq!(result.unwrap_err().to_string(), "oops")
    }

    fn make_db() -> Database {
        let sql_mock = SqlRunner::default();

        Database {
            sql_runner: sql_mock,
        }
    }

    fn make_db_with_mock<F>(create_mocks: F) -> Database
    where
        F: FnOnce(&mut MockSqlRunner),
    {
        let mut sql_mock = SqlRunner::default();
        create_mocks(&mut sql_mock);

        Database {
            sql_runner: sql_mock,
        }
    }
}
