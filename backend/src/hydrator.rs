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
    if plant.usda_source.is_none() {
        futures_unordered.push(Box::pin(hydrate_usda_source(plant)));
    }
    if plant.wiki_source.is_none() {
        futures_unordered.push(Box::pin(hydrate_wikipedia_source(plant)));
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
    let rating = match ai::fetch_pollinator_rating(&api_key, &plant.common).await {
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
    let rating = match ai::fetch_bird_rating(&api_key, &plant.common).await {
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
    let rating = match ai::fetch_animal_rating(&api_key, &plant.common).await {
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

async fn hydrate_usda_source(plant: &Plant) -> Option<HydratedPlant> {
    let usda_source = citations::usda::find(&plant.scientific)?;

    Some(HydratedPlant {
        updated: true,
        done: false,
        plant: Plant {
            usda_source: Some(usda_source),
            ..plant.clone()
        },
    })
}

async fn hydrate_wikipedia_source(plant: &Plant) -> Option<HydratedPlant> {
    let wiki_source = citations::wikipedia::find(&plant.scientific).await?;

    Some(HydratedPlant {
        updated: true,
        done: false,
        plant: Plant {
            wiki_source: Some(wiki_source),
            ..plant.clone()
        },
    })
}
