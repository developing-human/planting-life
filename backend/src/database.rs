use crate::domain::*;
use anyhow::anyhow;
use mockall::automock;
use mockall_double::double;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
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

    /// Fetches one Plant by id.  Returns None if it is not
    /// found or if there is a database error.
    pub async fn get_plant_by_id(&self, id: usize) -> Option<Plant> {
        match self.sql_runner.select_plant_by_id(id).await {
            Ok(Some(plant)) => Some(plant),
            Ok(None) => None,
            Err(e) => {
                warn!("get_plant_by_id failed to select: {e}");
                None
            }
        }
    }

    /// Fetches a garden by id.  The id may be the read_id or write_id.
    pub async fn get_garden(&self, id: &str) -> Option<Garden> {
        let mut garden = match self.sql_runner.select_garden_by_id(id, true).await {
            Ok(Some(garden)) => Some(garden),
            Ok(None) => None,
            Err(e) => {
                warn!("get_garden failed to select by read_id: {e}");
                None
            }
        };

        if garden.is_none() {
            garden = match self.sql_runner.select_garden_by_id(id, false).await {
                Ok(Some(garden)) => Some(Garden {
                    // get_garden was called with a write_id, so populate it
                    write_id: Some(id.to_string()),
                    ..garden
                }),
                Ok(None) => None,
                Err(e) => {
                    warn!("get_garden failed to select by write_id: {e}");
                    None
                }
            }
        }

        // Unwrap garden, returning None if it wasn't found
        let garden = garden?;

        let plants = match self
            .sql_runner
            .select_plants_by_garden_id(id, garden.write_id.is_none())
            .await
        {
            Ok(plants) => plants,
            Err(e) => {
                warn!("get_garden failed to select plants by garden id: {e}");
                vec![]
            }
        };

        Some(Garden { plants, ..garden })
    }

    /// Saves a new garden, returning the read_id and write_id.
    pub async fn save_new_garden(
        &self,
        garden: &Garden,
        plant_ids: Vec<usize>,
    ) -> anyhow::Result<(String, String)> {
        let read_id = self.get_unique_garden_id(5).await?;
        let write_id = self.get_unique_garden_id(20).await?;

        self.sql_runner
            .insert_garden(garden, &read_id, &write_id)
            .await
            .map_err(|e| anyhow!("save_new_garden failed: {e}"))?;

        match self
            .sql_runner
            .replace_garden_plants(&write_id, plant_ids)
            .await
        {
            Ok(()) => Ok((read_id, write_id)),
            Err(e) => Err(anyhow!("save_new_garden failed to replace plants: {e}")),
        }
    }

    /// Updates an existing garden, returning an empty result.
    pub async fn save_existing_garden(
        &self,
        write_id: &str,
        name: &str,
        plant_ids: Vec<usize>,
    ) -> anyhow::Result<()> {
        self.sql_runner
            .update_garden(write_id, name)
            .await
            .map_err(|e| anyhow!("save_existing_garden failed: {e}"))?;

        self.sql_runner
            .replace_garden_plants(write_id, plant_ids)
            .await
            .map_err(|e| anyhow!("save_new_garden failed to replace plants: {e}"))
    }

    /// Generates a unique garden id, ensuring it is not already used as an id
    /// for an existing Garden.
    async fn get_unique_garden_id(&self, length: u8) -> anyhow::Result<String> {
        let mut tries_remaining = 5;
        while tries_remaining > 0 {
            let proposed_id = generate_random_string(length);

            if self.get_garden(&proposed_id).await.is_none() {
                return Ok(proposed_id);
            } else {
                warn!("Collision when generating garden id w/ length={length}")
            }

            tries_remaining -= 1;
        }

        Err(anyhow!(
            "Ran out of tries generating garden read_id of length={length}"
        ))
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

    pub async fn find_plants_by_word_prefix(&self, word_prefix: &str) -> Vec<Plant> {
        if word_prefix.len() < 3 {
            return vec![];
        }

        let search_expression = build_word_prefix_expression(word_prefix);
        match self
            .sql_runner
            .find_plants_by_word_prefix(&search_expression)
            .await
        {
            Ok(plants) => plants,
            Err(e) => {
                warn!("find_plants_by_word_prefix failed to select: {e}");
                vec![]
            }
        }
    }
}

fn generate_random_string(length: u8) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

fn build_word_prefix_expression(word_prefix: &str) -> String {
    let mut expression = String::new();

    for word in word_prefix.split_whitespace() {
        if !word.is_empty() {
            expression.push_str(&format!("+{}* ", word));
        }
    }

    expression.trim().to_string()
}

#[cfg(test)]
mod tests {
    use mockall::Sequence;

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

    #[tokio::test]
    async fn test_generate_garden_id_first_try() {
        let db = make_db_with_mock(|mock| {
            mock.expect_select_garden_by_id().returning(|_, _| Ok(None));
        });

        let result = db.get_unique_garden_id(5).await;
        assert_eq!(result.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn test_generate_garden_id_third_try() {
        let db = make_db_with_mock(|mock| {
            let mut seq = Sequence::new();
            // Finds garden first two times
            mock.expect_select_garden_by_id()
                .times(2)
                .in_sequence(&mut seq)
                .returning(|_, _| {
                    Ok(Some(Garden::empty(
                        "name".to_string(),
                        "zip".to_string(),
                        Shade::Lots,
                        Moisture::Lots,
                    )))
                });

            // Doesn't find garden by read or write id on third attempt
            mock.expect_select_garden_by_id()
                .times(2)
                .in_sequence(&mut seq)
                .returning(|_, _| Ok(None));

            // Never finds plants, but thats ok.
            mock.expect_select_plants_by_garden_id()
                .returning(|_, _| Ok(vec![]));
        });

        let result = db.get_unique_garden_id(5).await;
        assert_eq!(result.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn test_generate_garden_id_out_of_tries() {
        let db = make_db_with_mock(|mock| {
            let mut seq = Sequence::new();
            // Finds garden first two times
            mock.expect_select_garden_by_id()
                .times(5)
                .in_sequence(&mut seq)
                .returning(|_, _| {
                    Ok(Some(Garden::empty(
                        "name".to_string(),
                        "zip".to_string(),
                        Shade::Lots,
                        Moisture::Lots,
                    )))
                });

            // Never finds plants, but thats ok.
            mock.expect_select_plants_by_garden_id()
                .returning(|_, _| Ok(vec![]));
        });

        let result = db.get_unique_garden_id(5).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_build_word_prefix_expression() {
        assert_eq!(build_word_prefix_expression("foo bar"), "+foo* +bar*");
        assert_eq!(build_word_prefix_expression("foo      bar"), "+foo* +bar*");
        assert_eq!(build_word_prefix_expression("  foo   bar "), "+foo* +bar*");
        assert_eq!(build_word_prefix_expression("  foo    "), "+foo*");
        assert_eq!(build_word_prefix_expression("  foobar    "), "+foobar*");
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
