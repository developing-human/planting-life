use std::collections::HashSet;

use crate::domain::*;
use futures::future;
use mysql_async::prelude::*;
use mysql_async::Pool;

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub fn new(url: &str) -> Self {
        Self {
            pool: Pool::new(url),
        }
    }

    pub async fn find_nurseries(&self, zip: &str) -> Vec<Nursery> {
        let mut conn = self.pool.get_conn().await.unwrap();

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
            .unwrap()
    }

    pub async fn lookup_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
    ) -> Vec<NativePlant> {
        let mut conn = self.pool.get_conn().await.unwrap();

        r"
SELECT p.id, p.scientific_name, p.common_name, p.description
FROM plants p
INNER JOIN queries_plants qp ON qp.plant_id = p.id
INNER JOIN queries q ON qp.query_id = q.id
INNER JOIN zipcodes z ON z.region_id = q.region_id
WHERE z.zipcode = ?
  AND q.moisture = ?
  AND q.shade = ?
  LIMIT 12
"
        .with((zip, moisture.to_string(), shade.to_string()))
        .map(&mut conn, |(id, scientific, common, description)| {
            NativePlant {
                id: Some(id),
                scientific,
                common,
                description,
                bloom: None, //TODO: Need to add a column for this
                image: None,
            }
        })
        .await
        .unwrap() //TODO: Need to more carefully think about error handling
    }

    pub async fn save_query_results(
        &self,
        zip: &str,
        moisture: &Moisture,
        shade: &Shade,
        all_plants: &Vec<NativePlant>,
        plants_to_save: &[NativePlant],
    ) {
        let mut conn = self.pool.get_conn().await.unwrap();

        // Convert to Vec<&NativePlant>
        let plants_to_save = plants_to_save.iter().collect();

        let saved_plants = self.save_plants(&plants_to_save).await;
        let query_id: i32 = r"INSERT INTO queries (moisture, shade, region_id) VALUES
            (?, ?, (SELECT region_id from zipcodes where zipcode = ?))
            RETURNING id"
            .with((moisture.to_string(), shade.to_string(), zip))
            .first(&mut conn)
            .await
            .unwrap() //TODO: Handle error
            .unwrap(); // Unwraps an option that will always be Some

        let mut plant_ids = HashSet::new();
        for plant in all_plants {
            if let Some(id) = plant.id {
                plant_ids.insert(id);
            }
        }
        for plant in saved_plants {
            if let Some(id) = plant.id {
                plant_ids.insert(id);
            }
        }

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
            .unwrap();
    }

    // Takes in a vector of plants which are not in the database (null ids), and
    // returns a new vector of native plants which have ids and are in the database
    async fn save_plants(&self, plants_in: &Vec<&NativePlant>) -> Vec<NativePlant> {
        let mut futures = vec![];
        for plant in plants_in {
            futures.push(self.save_plant(plant));
        }

        future::join_all(futures).await
    }

    // Takes in a plants which is not in the database, and returns a new plant with
    // its database id populated
    pub async fn save_plant(&self, plant: &NativePlant) -> NativePlant {
        let mut conn = self.pool.get_conn().await.unwrap();

        let id = r"INSERT INTO plants (scientific_name, common_name, description)
            VALUES (:scientific_name, :common_name, :description)
            RETURNING id"
            .with(params! {
                "scientific_name" => &plant.scientific,
                "common_name" => &plant.common,
                "description" => plant.description.clone().unwrap_or("null".to_string()),
            })
            .fetch(&mut conn)
            .await
            .unwrap();

        NativePlant {
            id: Some(id[0]),
            scientific: plant.scientific.clone(),
            common: plant.common.clone(),
            description: plant.description.clone(),
            bloom: None,
            image: None,
        }
    }

    pub async fn get_plant_by_id(&self, id: usize) -> NativePlant {
        let mut conn = self.pool.get_conn().await.unwrap();

        let (scientific, common, description) = r"
SELECT scientific_name, common_name, description
FROM plants
WHERE id = :plant_id"
            .with(params! {
                "plant_id" => id,
            })
            .first(&mut conn)
            .await
            .unwrap() // assume it worked
            .unwrap(); // assume it was found

        NativePlant {
            id: Some(id),
            scientific,
            common,
            description,
            bloom: None,
            image: None,
        }
    }

    pub async fn get_plant_by_scientific_name(&self, scientific_name: &str) -> Option<NativePlant> {
        let mut conn = self.pool.get_conn().await.unwrap();

        let query_result = r"
SELECT id, common_name, description
FROM plants
WHERE scientific_name = :scientific_name"
            .with(params! {
                "scientific_name" => scientific_name,
            })
            .first(&mut conn)
            .await
            .unwrap(); // assume it worked

        if let Some((id, common, description)) = query_result {
            Some(NativePlant {
                id: Some(id),
                scientific: scientific_name.to_string(),
                common,
                description,
                bloom: None,
                image: None,
            })
        } else {
            None
        }
    }
}
