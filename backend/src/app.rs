#[double]
use crate::database::Database;

use crate::{
    controllers::{
        gardens::{
            create_garden_handler, read_garden_handler, update_garden_handler, GardensController,
        },
        maps::{maps_api_key_handler, MapsController},
        nurseries::{fetch_nurseries_handler, NurseriesController},
        plants::{
            find_plant_handler, find_plants_handler, plants_stream_by_scientific_name_handler,
            plants_stream_handler, PlantController,
        },
    },
    highlights::Highlights,
};
use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use mockall_double::double;

pub struct PlantingLifeApp {
    pub gardens_controller: GardensController,
    pub plant_controller: PlantController,
    pub nursery_controller: NurseriesController,
    pub maps_controller: MapsController,
}

impl PlantingLifeApp {
    pub fn new(db_url: &str) -> Self {
        tracing_subscriber::fmt::init();

        let db = live_forever(Database::new(db_url));
        let highlights = live_forever(Highlights {});
        Self {
            gardens_controller: GardensController { db, highlights },
            plant_controller: PlantController { db, highlights },
            nursery_controller: NurseriesController { db },
            maps_controller: MapsController { db },
        }
    }

    pub async fn start(&'static self) -> std::io::Result<()> {
        println!("Starting!");
        HttpServer::new(move || {
            let mut cors = Cors::default()
                .allowed_origin("https://www.planting.life")
                .allowed_origin("https://planting.life")
                .allowed_origin("https://maps.planting.life")
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
                .service(plants_stream_handler)
                .service(find_plants_handler)
                .service(find_plant_handler)
                .service(fetch_nurseries_handler)
                .service(read_garden_handler)
                .service(create_garden_handler)
                .service(update_garden_handler)
                .service(maps_api_key_handler)
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
