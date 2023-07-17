use crate::database::Database;
use crate::domain::Plant;
use crate::flickr;
use crate::{ai, citations};
use futures::channel::mpsc::UnboundedSender;
use futures::stream::{FuturesUnordered, Stream, StreamExt};
use futures::Future;
use std::boxed::Box;
use std::env;
use std::pin::Pin;
use tracing::warn;

#[derive(Debug)]
pub struct HydratedPlant {
    pub plant: Plant,
    pub done: bool,
    pub updated: bool,
}

pub async fn hydrate_plants(
    db: &Database,
    mut plants: impl Stream<Item = Plant> + Unpin,
    sender: &mut UnboundedSender<HydratedPlant>,
) {
    // References to tasks which are running
    let mut handles = vec![];
    while let Some(plant) = plants.next().await {
        // Make a clone, so the inner and outer tasks can each own a sender
        let sender = sender.clone();
        let db = db.clone();

        // This inner task is started so the next entry can start processing before
        // the current one finishes.
        handles.push(actix_web::rt::spawn(async move {
            hydrate_one_plant(&db, plant, Some(sender)).await;
        }));
    }

    for handle in handles {
        handle.await.unwrap_or_default();
    }

    sender.close_channel();
}

/// Given a partially populated plant, populates it as best it can from
/// the database and LLM.  If a sender is provided, emits updates as
/// they become available.  The last plant emitted will be marked as done
/// and be populated as is possible.  Also Returns the fully populated
/// plant.
async fn hydrate_one_plant(
    db: &Database,
    mut plant: Plant,
    sender: Option<UnboundedSender<HydratedPlant>>,
) {
    // If this plant didn't come from the datbase, check the database for it.
    if plant.id.is_none() {
        let existing_plant = db.get_plant_by_scientific_name(&plant.scientific).await;
        if let Some(existing_plant) = existing_plant {
            plant = existing_plant;
        }
    }

    send_plant(&sender, &plant, false, false).await;

    // At this point I have a plant from the gpt list + database query
    // Some parts could be missing (not in db, db is missing parts)
    // Now, fill in any missing parts.

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
    futures_unordered.push(Box::pin(hydrate_details(&plant)));

    let mut updated = false;
    let mut merged_plant = plant.clone();
    while let Some(hydrated_plant) = futures_unordered.next().await {
        if let Some(hydrated_plant) = hydrated_plant {
            updated = true;
            merged_plant = merged_plant.merge(&hydrated_plant.plant);
            send_plant(&sender, &hydrated_plant.plant, false, true).await;
        }
    }

    send_plant(&sender, &merged_plant, true, updated).await;
}

/// Sends a HydratedPlant to the sender, if the sender is populated.
async fn send_plant(
    sender: &Option<UnboundedSender<HydratedPlant>>,
    plant: &Plant,
    done: bool,
    updated: bool,
) {
    if let Some(mut sender) = sender.clone() {
        sender
            .start_send(HydratedPlant {
                plant: plant.clone(),
                done,
                updated,
            })
            // This should only fail in the receiver is closed
            // and panicking seems okay in that scenario
            .unwrap();
    }
}

/// Looks up an image for this plant.  If one is found, it returns a HydratedPlant
/// with the image populated.
async fn hydrate_image(plant: &Plant) -> Option<HydratedPlant> {
    let flickr_api_key = env::var("FLICKR_API_KEY").expect("Must define $FLICKR_API_KEY");

    flickr::get_image(&plant.scientific, &plant.common, &flickr_api_key)
        .await
        .map(|image| HydratedPlant {
            updated: true,
            done: false,
            plant: Plant {
                image: Some(image),
                ..plant.clone()
            },
        })
}

/// Looks up a description for this plant.  If one is found, it returns a HydratedPlant
/// with the description populated.
async fn hydrate_description(plant: &Plant) -> Option<HydratedPlant> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let description_stream = match ai::fetch_description(&api_key, &plant.scientific).await {
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
            plant: Plant {
                description: Some(description_deltas.join("")),
                ..plant.clone()
            },
        })
    }
}

/// Hydrates all "details", which includes things like ratings, height,
/// width, and bloom season.
///
/// Returns None if none of those details are updated.
async fn hydrate_details(plant: &Plant) -> Option<HydratedPlant> {
    let mut futures_unordered: FuturesUnordered<
        Pin<Box<dyn Future<Output = Option<HydratedPlant>>>>,
    > = FuturesUnordered::new();

    if plant.pollinator_rating.is_none() {
        futures_unordered.push(Box::pin(hydrate_pollinator_rating(plant)));
    }
    if plant.bird_rating.is_none() {
        futures_unordered.push(Box::pin(hydrate_bird_rating(plant)));
    }
    if plant.animal_rating.is_none() {
        futures_unordered.push(Box::pin(hydrate_animal_rating(plant)));
    }
    if plant.citations.is_empty() {
        futures_unordered.push(Box::pin(hydrate_citations(plant)));
    }

    // Merges all fetched details into a single Plant before returning it
    let mut merged_plant: Option<Plant> = None;
    while let Some(hydrated_plant) = futures_unordered.next().await {
        if let Some(hydrated_plant) = hydrated_plant {
            if let Some(some_merged_plant) = merged_plant {
                merged_plant = Some(some_merged_plant.merge(&hydrated_plant.plant));
            } else {
                merged_plant = Some(hydrated_plant.plant.clone());
            }
        }
    }

    merged_plant.map(|plant| HydratedPlant {
        plant,
        updated: true,
        done: false,
    })
}

async fn hydrate_pollinator_rating(plant: &Plant) -> Option<HydratedPlant> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let rating = match ai::fetch_pollinator_rating(&api_key, &plant.scientific).await {
        Ok(stream) => stream,
        Err(e) => {
            warn!("Failed to fetch pollinator rating: {e}");
            return None;
        }
    };

    Some(HydratedPlant {
        updated: true,
        done: false,
        plant: Plant {
            pollinator_rating: Some(rating),
            ..plant.clone()
        },
    })
}

async fn hydrate_bird_rating(plant: &Plant) -> Option<HydratedPlant> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let rating = match ai::fetch_bird_rating(&api_key, &plant.scientific).await {
        Ok(stream) => stream,
        Err(e) => {
            warn!("Failed to fetch bird rating: {e}");
            return None;
        }
    };

    Some(HydratedPlant {
        updated: true,
        done: false,
        plant: Plant {
            bird_rating: Some(rating),
            ..plant.clone()
        },
    })
}
async fn hydrate_animal_rating(plant: &Plant) -> Option<HydratedPlant> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let rating = match ai::fetch_animal_rating(&api_key, &plant.scientific).await {
        Ok(stream) => stream,
        Err(e) => {
            warn!("Failed to fetch animal rating: {e}");
            return None;
        }
    };

    Some(HydratedPlant {
        updated: true,
        done: false,
        plant: Plant {
            animal_rating: Some(rating),
            ..plant.clone()
        },
    })
}

async fn hydrate_citations(plant: &Plant) -> Option<HydratedPlant> {
    //TODO: I think citations::find needs to know what citations we already have,
    //      and only try to build out the ones we don't have.  But currently we
    //      don't even have citations in the db.
    let citations = citations::find(&plant.scientific).await;
    if citations.is_empty() {
        None
    } else {
        Some(HydratedPlant {
            updated: true,
            done: false,
            plant: Plant {
                citations,
                ..plant.clone()
            },
        })
    }
}
