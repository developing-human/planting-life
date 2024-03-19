#[double]
use crate::ai::Ai;

#[double]
use crate::database::Database;

use crate::{
    ai::openai::OpenAI,
    citations::Citations,
    controllers::{
        gardens::{
            create_garden_handler, read_garden_handler, update_garden_handler, GardensController,
        },
        nurseries::{fetch_nurseries_handler, NurseriesController},
        plants::{
            find_plant_handler, find_plants_handler, plants_stream_by_scientific_name_handler,
            PlantController,
        },
    },
    flickr::Flickr,
    highlights::Highlights,
    hydrator::Hydrator,
    selector::Selector,
};
use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
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
        let highlights = live_forever(Highlights {});

        let hydrator = Hydrator::new(ai, flickr, citations, highlights);
        let selector = Selector::new(db, ai);

        Self {
            gardens_controller: GardensController { db, highlights },
            plant_controller: PlantController {
                db,
                hydrator,
                selector,
                highlights,
            },
            nursery_controller: NurseriesController { db },
        }
    }

    pub async fn start(&'static self) -> std::io::Result<()> {
        HttpServer::new(move || {
            let mut cors = Cors::default()
                .allowed_origin("https://www.planting.life")
                .allowed_origin("https://planting.life")
                .allowed_header(http::header::CONTENT_TYPE)
                .allowed_header(http::header::ACCEPT)
                .allowed_methods(vec!["GET", "POST", "PUT"]);

            // In local (debug build, not release), don't restrict origin
            // This allows localhost, but also networked locations (ex: access
            // from phone on local network)
            if cfg!(debug_assertions) {
                cors = cors.allow_any_origin()
            }

            App::new()
                .wrap(cors)
                .app_data(web::Data::new(self))
                .service(plants_stream_by_scientific_name_handler)
                .service(find_plants_handler)
                .service(find_plant_handler)
                .service(fetch_nurseries_handler)
                .service(read_garden_handler)
                .service(create_garden_handler)
                .service(update_garden_handler)
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
