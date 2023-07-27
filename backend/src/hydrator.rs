use crate::domain::Plant;
use crate::flickr;
use crate::{ai, citations};
use futures::channel::mpsc::UnboundedSender;
use futures::stream::{FuturesUnordered, Stream, StreamExt};
use futures::Future;
use std::boxed::Box;
use std::env;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::warn;

#[derive(Debug)]
pub struct HydratedPlant {
    pub plant: Plant,
    pub updated: bool,
}

pub const ALL_PLANTS_HYDRATING_MARKER: &str = "ALL PLANTS HYDRATING";

pub async fn hydrate_plants(
    mut plants: impl Stream<Item = Plant> + Unpin,
    sender: &mut UnboundedSender<HydratedPlant>,
) {
    // This semaphore only allows X plants to hydrate at once.  This provides
    // rate limiting so we don't crush the services we populate data from.
    let semaphore = Arc::new(Semaphore::new(6));

    // References to tasks which are running
    let mut handles = vec![];

    while let Some(plant) = plants.next().await {
        // Make clones to share with the async task
        let sender = sender.clone();
        let permit = Arc::clone(&semaphore);

        // This inner task is started so the next entry can start processing before
        // the current one finishes.
        handles.push(actix_web::rt::spawn(async move {
            // Don't hydrate until a permit is available.
            let _permit = permit.acquire().await.unwrap();

            hydrate_one_plant(plant, Some(sender)).await;
        }));
    }

    // Alright... this is janky... I was having a hard time getting an async callback
    // to work, so I'm sending back a "marker plant" which tells the caller that we've
    // started hydrating all plants.  This ultimately lets the front end know it can
    // stop showing the large loading symbol.
    send_plant(
        &Some(sender.clone()),
        &Plant::new(ALL_PLANTS_HYDRATING_MARKER, ALL_PLANTS_HYDRATING_MARKER),
        false,
        false,
    )
    .await;

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
async fn hydrate_one_plant(plant: Plant, sender: Option<UnboundedSender<HydratedPlant>>) {
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
    futures_unordered.push(Box::pin(hydrate_ratings(&plant)));
    futures_unordered.push(Box::pin(hydrate_citations(&plant)));
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
                plant: Plant {
                    done_loading: done,
                    ..plant.clone()
                },
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
            plant: Plant {
                image: Some(image),
                ..plant.clone()
            },
        })
}

/// Hydrates all "details", which includes things like height,
/// width, and bloom season.
///
/// Returns None if none of those details are updated.
async fn hydrate_details(plant: &Plant) -> Option<HydratedPlant> {
    let futures_unordered: FuturesUnordered<Pin<Box<dyn Future<Output = Option<HydratedPlant>>>>> =
        FuturesUnordered::new();

    if plant.height.is_none() {
        futures_unordered.push(Box::pin(hydrate_height(plant.clone())));
    }
    if plant.spread.is_none() {
        futures_unordered.push(Box::pin(hydrate_spread(plant.clone())));
    }
    if plant.bloom.is_none() {
        futures_unordered.push(Box::pin(hydrate_bloom(plant.clone())));
    }

    merge_hydrated_plants(futures_unordered).await
}

async fn hydrate_ratings(plant: &Plant) -> Option<HydratedPlant> {
    let futures_unordered: FuturesUnordered<Pin<Box<dyn Future<Output = Option<HydratedPlant>>>>> =
        FuturesUnordered::new();

    if plant.pollinator_rating.is_none() {
        futures_unordered.push(Box::pin(hydrate_pollinator_rating(plant.clone())));
    }
    if plant.bird_rating.is_none() {
        futures_unordered.push(Box::pin(hydrate_bird_rating(plant.clone())));
    }
    if plant.animal_rating.is_none() {
        futures_unordered.push(Box::pin(hydrate_animal_rating(plant.clone())));
    }

    merge_hydrated_plants(futures_unordered).await
}

async fn hydrate_citations(plant: &Plant) -> Option<HydratedPlant> {
    let futures_unordered: FuturesUnordered<Pin<Box<dyn Future<Output = Option<HydratedPlant>>>>> =
        FuturesUnordered::new();

    if plant.usda_source.is_none() {
        futures_unordered.push(Box::pin(hydrate_usda_source(plant.clone())));
    }
    if plant.wiki_source.is_none() {
        futures_unordered.push(Box::pin(hydrate_wikipedia_source(plant.clone())));
    }

    merge_hydrated_plants(futures_unordered).await
}

/// Merges all fetched details into a single Plant before returning it
async fn merge_hydrated_plants(
    mut futures_unordered: FuturesUnordered<Pin<Box<dyn Future<Output = Option<HydratedPlant>>>>>,
) -> Option<HydratedPlant> {
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
    })
}

async fn hydrate_pollinator_rating(plant: Plant) -> Option<HydratedPlant> {
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
        plant: Plant {
            pollinator_rating: Some(rating),
            ..plant
        },
    })
}

async fn hydrate_bird_rating(plant: Plant) -> Option<HydratedPlant> {
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
        plant: Plant {
            bird_rating: Some(rating),
            ..plant
        },
    })
}
async fn hydrate_animal_rating(plant: Plant) -> Option<HydratedPlant> {
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
        plant: Plant {
            animal_rating: Some(rating),
            ..plant
        },
    })
}

async fn hydrate_usda_source(plant: Plant) -> Option<HydratedPlant> {
    let usda_source = citations::usda::find(&plant.scientific)?;

    Some(HydratedPlant {
        updated: true,
        plant: Plant {
            usda_source: Some(usda_source),
            ..plant
        },
    })
}

async fn hydrate_wikipedia_source(plant: Plant) -> Option<HydratedPlant> {
    let wiki_source = citations::wikipedia::find(&plant.scientific).await?;

    Some(HydratedPlant {
        updated: true,
        plant: Plant {
            wiki_source: Some(wiki_source),
            ..plant
        },
    })
}

async fn hydrate_height(plant: Plant) -> Option<HydratedPlant> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let height = match ai::fetch_height(&api_key, &plant.common).await {
        Ok(stream) => stream,
        Err(e) => {
            warn!("Failed to fetch height: {e}");
            return None;
        }
    };

    Some(HydratedPlant {
        updated: true,
        plant: Plant {
            height: Some(height),
            ..plant
        },
    })
}

async fn hydrate_spread(plant: Plant) -> Option<HydratedPlant> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let spread = match ai::fetch_spread(&api_key, &plant.common).await {
        Ok(stream) => stream,
        Err(e) => {
            warn!("Failed to fetch spread: {e}");
            return None;
        }
    };

    Some(HydratedPlant {
        updated: true,
        plant: Plant {
            spread: Some(spread),
            ..plant
        },
    })
}

async fn hydrate_bloom(plant: Plant) -> Option<HydratedPlant> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let bloom = match ai::fetch_bloom(&api_key, &plant.common).await {
        Ok(stream) => stream,
        Err(e) => {
            warn!("Failed to fetch bloom: {e}");
            return None;
        }
    };

    Some(HydratedPlant {
        updated: true,
        plant: Plant {
            bloom: Some(bloom),
            ..plant
        },
    })
}
