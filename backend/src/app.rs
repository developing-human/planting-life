#[double]
use crate::ai::Ai;

#[double]
use crate::database::Database;

use crate::{
    ai::openai::OpenAI,
    citations::Citations,
    controllers::{
        gardens::{create_garden_handler, read_garden_handler, GardensController},
        nurseries::{fetch_nurseries_handler, NurseriesController},
        plants::{fetch_plants_handler, PlantController},
    },
    flickr::Flickr,
    highlights::Highlights,
    hydrator::Hydrator,
    selector::Selector,
};
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use mockall_double::double;

pub struct PlantingLifeApp {
    pub gardens_controller: GardensController,
    pub plant_controller: PlantController,
    pub nursery_controller: NurseriesController,
}

impl PlantingLifeApp {
    pub fn new(db_url: &str, openai_api_key: &str, flickr_api_key: &str) -> Self {
        tracing_subscriber::fmt::init();

        let db = live_forever(Database::new(db_url));
        let open_ai = OpenAI::new(openai_api_key.into());
        let flickr = Flickr::new(flickr_api_key.into());

        let ai = live_forever(Ai::new(open_ai));

        let citations = Citations {};
        let highlights = Highlights {};

        let hydrator = Hydrator::new(ai, flickr, citations, highlights);
        let selector = Selector::new(db, ai);

        let highlights = Highlights {};

        Self {
            gardens_controller: GardensController { db, highlights },
            plant_controller: PlantController {
                db,
                hydrator,
                selector,
            },
            nursery_controller: NurseriesController { db },
        }
    }

    pub async fn start(&'static self) -> std::io::Result<()> {
        HttpServer::new(move || {
            let cors = Cors::default()
                .allowed_origin("https://www.planting.life")
                .allowed_origin("https://planting.life")
                .allowed_methods(vec!["GET"]);

            App::new()
                .wrap(cors)
                .app_data(web::Data::new(self))
                .service(fetch_plants_handler)
                .service(fetch_nurseries_handler)
                .service(read_garden_handler)
                .service(create_garden_handler)
        })
        .bind("0.0.0.0:8080")?
        .run()
        .await
    }
}

// When building the app its often necessary for Rust to know
// components will live for the duration of the application.
// The "leaks" them to get a static reference.
fn live_forever<T>(to_leak: T) -> &'static T {
    Box::leak(Box::new(to_leak))
}
