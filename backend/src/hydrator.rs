#[double]
use crate::ai::Ai;
use crate::citations::Citations;
use crate::domain::Plant;
use crate::flickr::Flickr;
use crate::highlights::Highlights;
use futures::channel::mpsc::UnboundedSender;
use futures::stream::{FuturesUnordered, Stream, StreamExt};
use futures::Future;
use mockall_double::double;
use std::boxed::Box;
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

pub struct Hydrator {
    ai: &'static Ai,
    flickr: Flickr,
    citations: Citations,
    highlights: Highlights,
}

impl Hydrator {
    pub fn new(
        ai: &'static Ai,
        flickr: Flickr,
        citations: Citations,
        highlights: Highlights,
    ) -> Self {
        Self {
            ai,
            flickr,
            citations,
            highlights,
        }
    }

    pub async fn hydrate_plants(
        &'static self,
        mut plants: Pin<Box<dyn Stream<Item = Plant> + Send>>,
        sender: &mut UnboundedSender<HydratedPlant>,
    ) {
        // This semaphore only allows X plants to hydrate at once.  This provides
        // rate limiting so we don't crush the services we populate data from.
        let semaphore = Arc::new(Semaphore::new(3));

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

                self.hydrate_one_plant(plant, Some(sender)).await;
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
    async fn hydrate_one_plant(
        &'static self,
        plant: Plant,
        sender: Option<UnboundedSender<HydratedPlant>>,
    ) {
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
            futures_unordered.push(Box::pin(self.hydrate_image(&plant)));
        }
        futures_unordered.push(Box::pin(self.hydrate_ratings(&plant)));
        futures_unordered.push(Box::pin(self.hydrate_citations(&plant)));
        futures_unordered.push(Box::pin(self.hydrate_details(&plant)));

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

    /// Looks up an image for this plant.  If one is found, it returns a HydratedPlant
    /// with the image populated.
    async fn hydrate_image(&self, plant: &Plant) -> Option<HydratedPlant> {
        self.flickr
            .get_image(&plant.scientific, &plant.common)
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
    async fn hydrate_details(&'static self, plant: &Plant) -> Option<HydratedPlant> {
        let futures_unordered: FuturesUnordered<
            Pin<Box<dyn Future<Output = Option<HydratedPlant>>>>,
        > = FuturesUnordered::new();

        if plant.height.is_none() {
            futures_unordered.push(Box::pin(self.hydrate_height(plant.clone())));
        }
        if plant.spread.is_none() {
            futures_unordered.push(Box::pin(self.hydrate_spread(plant.clone())));
        }
        if plant.bloom.is_none() {
            futures_unordered.push(Box::pin(self.hydrate_bloom(plant.clone())));
        }

        merge_hydrated_plants(futures_unordered).await
    }

    async fn hydrate_ratings(&'static self, plant: &Plant) -> Option<HydratedPlant> {
        let futures_unordered: FuturesUnordered<
            Pin<Box<dyn Future<Output = Option<HydratedPlant>>>>,
        > = FuturesUnordered::new();

        if plant.pollinator_rating.is_none() {
            futures_unordered.push(Box::pin(self.hydrate_pollinator_rating(plant.clone())));
        }
        if plant.bird_rating.is_none() {
            futures_unordered.push(Box::pin(self.hydrate_bird_rating(plant.clone())));
        }
        if plant.spread_rating.is_none() {
            futures_unordered.push(Box::pin(self.hydrate_spread_rating(plant.clone())));
        }
        if plant.deer_resistance_rating.is_none() {
            futures_unordered.push(Box::pin(self.hydrate_deer_resistance_rating(plant.clone())));
        }

        merge_hydrated_plants(futures_unordered)
            .await
            .or_else(|| {
                // If there was nothing to merge (ratings were already populated)
                // then make a HydratedPlant which wraps the original plant so the
                // highlights can be generated
                Some(HydratedPlant {
                    updated: false,
                    plant: plant.clone(),
                })
            })
            .map(|mut merged| {
                merged.plant.highlights = self.highlights.generate(&merged.plant);
                merged
            })
    }

    async fn hydrate_citations(&'static self, plant: &Plant) -> Option<HydratedPlant> {
        let futures_unordered: FuturesUnordered<
            Pin<Box<dyn Future<Output = Option<HydratedPlant>>>>,
        > = FuturesUnordered::new();

        if plant.usda_source.is_none() {
            futures_unordered.push(Box::pin(self.hydrate_usda_source(plant.clone())));
        }
        if plant.wiki_source.is_none() {
            futures_unordered.push(Box::pin(self.hydrate_wikipedia_source(plant.clone())));
        }

        merge_hydrated_plants(futures_unordered).await
    }

    async fn hydrate_pollinator_rating(&self, plant: Plant) -> Option<HydratedPlant> {
        let rating = match self.ai.fetch_pollinator_rating(&plant.common).await {
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

    async fn hydrate_bird_rating(&self, plant: Plant) -> Option<HydratedPlant> {
        let rating = match self.ai.fetch_bird_rating(&plant.common).await {
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

    async fn hydrate_spread_rating(&self, plant: Plant) -> Option<HydratedPlant> {
        let rating = match self.ai.fetch_spread_rating(&plant.common).await {
            Ok(rating) => rating,
            Err(e) => {
                warn!("Failed to fetch spread rating: {e}");
                return None;
            }
        };

        Some(HydratedPlant {
            updated: true,
            plant: Plant {
                spread_rating: Some(rating),
                ..plant
            },
        })
    }

    async fn hydrate_deer_resistance_rating(&self, plant: Plant) -> Option<HydratedPlant> {
        let rating = match self.ai.fetch_deer_resistance_rating(&plant.common).await {
            Ok(rating) => rating,
            Err(e) => {
                warn!("Failed to fetch spread rating: {e}");
                return None;
            }
        };

        Some(HydratedPlant {
            updated: true,
            plant: Plant {
                deer_resistance_rating: Some(rating),
                ..plant
            },
        })
    }

    async fn hydrate_usda_source(&self, plant: Plant) -> Option<HydratedPlant> {
        let usda_source = self.citations.find_usda(&plant.scientific)?;

        Some(HydratedPlant {
            updated: true,
            plant: Plant {
                usda_source: Some(usda_source),
                ..plant
            },
        })
    }

    async fn hydrate_wikipedia_source(&self, plant: Plant) -> Option<HydratedPlant> {
        let wiki_source = self.citations.find_wikipedia(&plant.scientific).await?;

        Some(HydratedPlant {
            updated: true,
            plant: Plant {
                wiki_source: Some(wiki_source),
                ..plant
            },
        })
    }

    async fn hydrate_height(&self, plant: Plant) -> Option<HydratedPlant> {
        let height = match self.ai.fetch_height(&plant.common).await {
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

    async fn hydrate_spread(&self, plant: Plant) -> Option<HydratedPlant> {
        let spread = match self.ai.fetch_spread(&plant.common).await {
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

    async fn hydrate_bloom(&self, plant: Plant) -> Option<HydratedPlant> {
        let bloom = match self.ai.fetch_bloom(&plant.common).await {
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
