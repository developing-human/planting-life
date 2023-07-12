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
                image: if img_id.is_some() {
                    Some(Image {
                        id: img_id,
                        scientific_name: scientific,
                        title: title.unwrap(),
                        card_url: card_url.unwrap(),
                        original_url: original_url.unwrap(),
                        author: author.unwrap(),
                        license: license.unwrap(),
                        license_url: "TODO".to_string(),
                    })
                } else {
                    None
                },
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
        plant_ids: HashSet<usize>,
    ) {
        let mut conn = self.pool.get_conn().await.unwrap();

        let query_id: i32 = r"INSERT INTO queries (moisture, shade, region_id) VALUES
            (?, ?, (SELECT region_id from zipcodes where zipcode = ?))
            RETURNING id"
            .with((moisture.to_string(), shade.to_string(), zip))
            .first(&mut conn)
            .await
            .unwrap() //TODO: Handle error
            .unwrap(); // Unwraps an option that will always be Some

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
    pub async fn save_plants(&self, plants_in: &Vec<&NativePlant>) -> Vec<NativePlant> {
        let mut futures = vec![];
        for plant in plants_in {
            futures.push(self.save_plant(plant));
        }

        future::join_all(futures).await
    }

    // Takes in a plant which may or may not be in the database, and returns
    // a new plant with its database id populated
    pub async fn save_plant(&self, plant: &NativePlant) -> NativePlant {
        let mut conn = self.pool.get_conn().await.unwrap();

        let mut img_id = None;
        if let Some(image) = &plant.image {
            img_id = image.id;
            if image.id.is_none() {
                let saved_image = self.save_image(image).await;
                img_id = saved_image.id;
            }
        }

        let id = if let Some(id) = plant.id {
            println!("UPDATING PLANT: {}", plant.scientific);
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
                .unwrap();

            id
        } else {
            println!("INSERTING PLANT: {}", plant.scientific);
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
                .unwrap()[0]
        };

        NativePlant {
            id: Some(id),
            scientific: plant.scientific.clone(),
            common: plant.common.clone(),
            description: plant.description.clone(),
            bloom: None,
            image: None,
        }
    }

    async fn save_image(&self, image: &Image) -> Image {
        let mut conn = self.pool.get_conn().await.unwrap();

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
            .unwrap();

        Image {
            id: Some(id[0]),
            title: image.title.clone(),
            card_url: image.card_url.clone(),
            original_url: image.original_url.clone(),
            author: image.author.clone(),
            license: image.license.clone(),
            scientific_name: image.scientific_name.clone(),
            license_url: image.license_url.clone(),
        }
    }

    //TODO: Maybe not needed
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
SELECT p.id, p.common_name, p.bloom, p.description, i.id, i.title, i.card_url, i.original_url, i.author, i.license
FROM plants p
INNER JOIN images i ON i.id = p.image_id
WHERE scientific_name = :scientific_name"
            .with(params! {
                "scientific_name" => scientific_name,
            })
            .first(&mut conn)
            .await
            .unwrap(); // assume it worked

        if let Some((
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
        )) = query_result
        {
            // Type information is needed below, and other attempts weren't working :)
            let img_id: Option<usize> = img_id;

            Some(NativePlant {
                id: Some(id),
                scientific: scientific_name.to_string(),
                common,
                description,
                bloom,
                image: if img_id.is_some() {
                    Some(Image {
                        id: img_id,
                        scientific_name: scientific_name.to_string(),
                        title,
                        card_url,
                        original_url,
                        author,
                        license,
                        license_url: "TODO".to_string(),
                    })
                } else {
                    None
                },
            })
        } else {
            None
        }
    }
}
