use actix_web::{get, web, Responder};
use std::time::Duration;

use actix_web_lab::sse::{self, ChannelStream, Sender, Sse};
use futures::{channel::mpsc, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::log::{info, warn};

use crate::{app::PlantingLifeApp, database::Database, domain::*, hydrator, selector};

#[derive(Serialize, Deserialize, Debug)]
struct PlantsRequest {
    zip: String,
    shade: Shade,
    moisture: Moisture,
}

pub struct PlantController {
    pub db: &'static dyn Database,
}

impl PlantController {
    pub fn new(db: &'static dyn Database) -> Self {
        Self { db }
    }

    async fn stream(
        &'static self,
        payload: PlantsRequest,
    ) -> Result<impl Responder, actix_web::Error> {
        info!("{payload:?}");

        let valid_zip = self.get_closest_valid_zip(&payload.zip).await?;
        let valid_payload = PlantsRequest {
            zip: valid_zip,
            ..payload
        };
        drop(payload); // Don't use the unvalidated payload by mistake

        let (frontend_sender, stream): (Sender, Sse<ChannelStream>) = sse::channel(10);

        // The real work is done in a new thread so the connection to the front end can stay open.
        actix_web::rt::spawn(async move {
            let plant_stream = selector::stream_plants(
                self.db,
                &valid_payload.zip,
                &valid_payload.moisture,
                &valid_payload.shade,
            )
            .await;

            match plant_stream {
                Ok(plant_stream) => {
                    self.hydrate_and_send_plants(
                        &valid_payload,
                        plant_stream.stream,
                        &frontend_sender,
                        plant_stream.from_db,
                    )
                    .await
                }
                Err(e) => {
                    warn!("error getting plant stream: {e}");
                    send_event(&frontend_sender, "error", "").await
                }
            };

            send_event(&frontend_sender, "close", "").await;
        });

        Ok(stream
            .with_keep_alive(Duration::from_secs(1))
            .customize()
            .insert_header(("X-Accel-Buffering", "no")))
    }

    async fn get_closest_valid_zip(&self, zip: &str) -> Result<String, actix_web::Error> {
        let valid_zip = self.db.lookup_closest_valid_zip(zip).await.map_err(|e| {
            warn!("Cannot find valid zipcode: {e}");
            actix_web::error::ErrorBadRequest("invalid zipcode")
        })?;

        if valid_zip != zip {
            info!("Adjusted unknown zip {} to {valid_zip}", zip);
        }

        Ok(valid_zip)
    }

    async fn hydrate_and_send_plants(
        &self,
        payload: &PlantsRequest,
        plant_stream: impl Stream<Item = Plant> + 'static + Unpin,
        frontend_sender: &Sender,
        plants_from_db: bool,
    ) {
        let (mut plant_sender, mut plant_receiver) = mpsc::unbounded();

        actix_web::rt::spawn(async move {
            hydrator::hydrate_plants(plant_stream, &mut plant_sender).await;
        });
        //let mut plants_to_save = vec![];
        let mut all_plants = vec![];
        let mut saved_plants = vec![];
        while let Some(hydrated_plant) = plant_receiver.next().await {
            if hydrated_plant.plant.scientific == hydrator::ALL_PLANTS_HYDRATING_MARKER {
                send_event(frontend_sender, "allPlantsLoaded", "").await;

                continue;
            }

            send_plant(frontend_sender, &hydrated_plant.plant).await;

            if hydrated_plant.plant.done_loading {
                all_plants.push(hydrated_plant.plant.clone());

                if hydrated_plant.updated {
                    //plants_to_save.push(hydrated_plant.plant.clone());
                    match self.db.save_plant(&hydrated_plant.plant).await {
                        Ok(plant) => saved_plants.push(plant.clone()),
                        Err(e) => warn!(
                            "Failed to save {} due to {e}",
                            hydrated_plant.plant.scientific
                        ),
                    }
                }
            }
        }

        // We only need to cache the query results if these results aren't from the database
        // When they are from the database, we know its already there.
        if !plants_from_db {
            self.db
                .save_query_results(
                    &payload.zip,
                    &payload.moisture,
                    &payload.shade,
                    all_plants,
                    saved_plants,
                )
                .await;
        }
    }
}

async fn send_plant(sender: &Sender, plant: &Plant) {
    let json = serde_json::to_string(&plant).expect("plant should serialize");

    send_event(sender, "plant", &json).await;
}

async fn send_event(sender: &Sender, event: &str, message: &str) {
    let data = sse::Data::new(message).event(event);

    match sender.send(data).await {
        Ok(_) => {}
        Err(_) => warn!("Error sending [{}] with message [{}]", event, message),
    }
}

#[get("/plants")]
async fn fetch_plants_handler(
    web::Query(payload): web::Query<PlantsRequest>,
    app: web::Data<&'static PlantingLifeApp>,
) -> Result<impl Responder, actix_web::Error> {
    app.plant_controller.stream(payload).await
}
