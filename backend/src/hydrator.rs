use crate::database::Database;
use crate::domain::NativePlant;
use crate::flickr;
use crate::openai;
use futures::stream::{FuturesUnordered, Stream, StreamExt};
use futures::Future;
use std::boxed::Box;
use std::env;
use std::pin::Pin;
use std::sync::mpsc::Sender;
use tracing::warn;

pub struct HydratedPlant {
    pub plant: NativePlant,
    pub done: bool,
    pub updated: bool,
}

pub async fn hydrate_plants(
    db: &Database,
    mut plants: impl Stream<Item = NativePlant> + Unpin,
    sender: &mut Sender<HydratedPlant>,
) {
    // Holds (and owns) all the plants which are returned.
    //let all_plants = Arc::new(Mutex::new(vec![]));

    // Holds references to plants which are either new or updated
    //let plants_to_save = Arc::new(Mutex::new(vec![]));

    // References to tasks which are running
    let mut handles = vec![];
    while let Some(plant) = plants.next().await {
        // Make a clone, so the inner and outer tasks can each own a sender
        let mut sender = sender.clone();
        let db = db.clone();

        //let all_plants = all_plants.clone();
        //let plants_to_save = plants_to_save.clone();

        // This inner task is started so the next entry can start processing before
        // the current one finishes.
        let handle = actix_web::rt::spawn(async move {
            hydrate_one_plant(&db, plant, &mut sender).await;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap_or_default();
    }
}

async fn hydrate_one_plant(
    db: &Database,
    mut plant: NativePlant,
    sender: &mut Sender<HydratedPlant>,
) {
    // If this plant didn't come from the datbase, check the database for it.
    if plant.id.is_none() {
        let existing_plant = db.get_plant_by_scientific_name(&plant.scientific).await;
        if let Some(existing_plant) = existing_plant {
            plant = existing_plant;
        }
    }

    // At this point I have a plant from the gpt list + database query
    // Some parts could be missing (not in db, db is missing parts)
    // Now, any missing parts need to be filled in.

    // This ridiculousness is to handle the fact that each async fn returns
    // a unique type, even if they return the same concrete type.  The "dyn"
    // specifically helps cover that up.
    let mut futures_unordered: FuturesUnordered<
        Pin<Box<dyn Future<Output = Option<HydratedPlant>>>>,
    > = FuturesUnordered::new();

    if plant.image.is_none() {
        futures_unordered.push(Box::pin(hydrate_image(&plant)));
    }
    if plant.description.is_none() {
        futures_unordered.push(Box::pin(hydrate_description(&plant)));
    }
    //TODO: This section will grow when more fields are added

    let mut updated = false;
    let mut merged_plant = plant.clone();
    while let Some(hydrated_plant) = futures_unordered.next().await {
        if let Some(hydrated_plant) = hydrated_plant {
            updated = true;
            merged_plant = merged_plant.merge(&hydrated_plant.plant);
            send_plant(sender, &hydrated_plant.plant, false, true).await;
        }
    }

    send_plant(sender, &merged_plant, updated, true).await;
}
/*
async fn fill_and_send_plants(
    db: web::Data<Database>,
    payload: &PlantsRequest,
    plants: impl Stream<Item = NativePlant>,
    sender: &Sender,
    plants_from_db: bool,
) {
    let db = web::Data::new(Arc::new(db));

    // Holds (and owns) all the plants which are returned.
    let all_plants = Arc::new(Mutex::new(vec![]));

    // Holds references to plants which are either new or updated
    let plants_to_save = Arc::new(Mutex::new(vec![]));

    // References to tasks which are running
    let mut handles = vec![];

    let mut plants = Box::pin(plants);

    while let Some(plant) = plants.next().await {
        // Make a clone, so the inner and outer tasks can each own a sender
        let sender_clone = sender.clone();
        let db = db.get_ref().clone();

        let all_plants = all_plants.clone();
        let plants_to_save = plants_to_save.clone();

        // This inner task is started so the next entry can start processing before
        // the previous one finishes.
        let handle = actix_web::rt::spawn(async move {
            // If this plant didn't come from the datbase, check the database for it.
            let mut plant = plant;
            if plant.id.is_none() {
                let existing_plant = db.get_plant_by_scientific_name(&plant.scientific).await;
                if let Some(existing_plant) = existing_plant {
                    plant = existing_plant;
                }
            }

            // At this point I have a plant from the gpt list + database query
            // Some parts could be missing (not in db, db is missing parts)
            // Now, any missing parts need to be filled in.

            // Concurrently send the plant to the front end while handling the image
            let (_, img, description /*, _*/) = join!(
                send_plant(&sender_clone, &plant),
                fetch_and_send_image(&sender_clone, &plant),
                fetch_and_send_description(&sender_clone, &plant),
                //TODO: Bring citations back once they can be cached or displayed.
                //fetch_and_send_citations(&sender_clone, &plant),
            );

            let updated_plant = NativePlant {
                image: img,
                description,
                ..plant
            };

            // Only save plants which weren't from the database (no id) or where
            // the description was updated.  In the future, we'll also want to check
            // images and citations once those are handled by the db.
            if updated_plant.id.is_none()
                || updated_plant.description != plant.description
                || (plant.image.is_none() && updated_plant.image.is_some())
            {
                plants_to_save.lock().await.push(updated_plant.clone());
            }

            all_plants.lock().await.push(updated_plant);
        });

        handles.push(handle);
    }

    send_event(sender, "allPlantsLoaded", "").await;

    // Wait for all inner tasks to finish before closing the stream
    // This lets any outstanding data be written back to the client
    for handle in handles {
        handle.await.unwrap_or_default();
    }

    let plants_to_save = plants_to_save.lock().await;
    let all_plants = all_plants.lock().await;

    // Save any plants which are new or updated.  If any fail, don't cache the query results.
    // This is because missing ids will result in a partial cache.
    let saved_plants = match db.save_plants(&plants_to_save.iter().collect()).await {
        Ok(saved_plants) => saved_plants,
        Err(e) => {
            warn!("failed to save plants, not caching: {e}");
            return;
        }
    };

    // We only need to cache the query results if these results aren't from the database
    // When they are from the database, we know its already there.
    if plants_from_db {
        return; // not logging, this is very common
    }

    // Also, don't save the results of this query if we have fewer than the desired number,
    // this should be a rare occurance and this is a simple way to handle it.  The
    // alternative would be to (on fetch) get some from the database and the rest from gpt.
    // Its easier to just get all from gpt, even if its a little more work.
    if all_plants.len() != 12 {
        info!("only have {} plants, not caching", all_plants.len());
        return;
    }

    // At least one plant is missing an image, so don't store these results.  Very
    // occasionally we'll run into this, and its okay as a quirk but lets not cache
    // it forever.
    let plant_without_image = all_plants.iter().find(|p| p.image.is_none());
    if let Some(plant_without_image) = plant_without_image {
        info!(
            "not all plants have an image (missing for {}), not caching",
            plant_without_image.scientific
        );
        return;
    }

    let plant_ids: HashSet<usize> = all_plants
        .iter()
        .chain(saved_plants.iter())
        .filter_map(|p| p.id)
        .collect();

    db.save_query_results(&payload.zip, &payload.moisture, &payload.shade, plant_ids)
        .await;
}
*/

async fn send_plant(
    sender: &mut Sender<HydratedPlant>,
    plant: &NativePlant,
    done: bool,
    updated: bool,
) {
    sender
        .send(HydratedPlant {
            plant: plant.clone(),
            done,
            updated,
        })
        // This should only fail in the receiver is closed
        // and panicking seems okay in that scenario
        .unwrap();
}

// Looks up an image for this plant.  If one is found, it returns a HydratedPlant
// with the image populated.
async fn hydrate_image(plant: &NativePlant) -> Option<HydratedPlant> {
    let flickr_api_key = env::var("FLICKR_API_KEY").expect("Must define $FLICKR_API_KEY");

    flickr::get_image(&plant.scientific, &plant.common, &flickr_api_key)
        .await
        .map(|image| HydratedPlant {
            updated: true,
            done: false,
            plant: NativePlant {
                image: Some(image),
                ..plant.clone()
            },
        })
}

async fn hydrate_description(plant: &NativePlant) -> Option<HydratedPlant> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let description_stream = match openai::fetch_description(&api_key, &plant.scientific).await {
        Ok(stream) => stream,
        Err(_) => {
            warn!("Failed to fetch description");
            return None;
        }
    };

    let mut description_deltas = vec![];
    let mut description_stream = Box::pin(description_stream);
    while let Some(description_delta) = description_stream.next().await {
        description_deltas.push(description_delta);
    }

    if description_deltas.is_empty() {
        None
    } else {
        Some(HydratedPlant {
            updated: true,
            done: false,
            plant: NativePlant {
                description: Some(description_deltas.join("")),
                ..plant.clone()
            },
        })
    }
}

/*
async fn fetch_and_send_citations(sender: &Sender, plant: &NativePlant) {
    //TODO: I think citations::find needs to know what citations we already have,
    //      and only try to build out the ones we don't have.  But currently we
    //      don't even have citations in the db.
    let citations = citations::find(&plant.scientific).await;
    if !citations.is_empty() {
        let citations_json = serde_json::to_string(&citations).expect("citations should serialize");
        let payload = format!(
            r#"{{"scientificName": "{}", "citations": {}}}"#,
            plant.scientific, citations_json
        );
        send_event(sender, "citations", &payload).await;
    }
}
*/
