use std::{env, sync::Arc};

#[double]
use crate::database::Database;

use crate::{
    controllers::{
        gardens::{
            create_garden_handler, list_garden_handler, read_garden_handler, update_garden_handler,
            GardensController,
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

        let db = Arc::new(Database::new(db_url));
        let highlights = Arc::new(Highlights {});
        Self {
            gardens_controller: GardensController {
                db: db.clone(),
                highlights: highlights.clone(),
            },
            plant_controller: PlantController {
                db: db.clone(),
                highlights: highlights.clone(),
            },
            nursery_controller: NurseriesController { db: db.clone() },
            maps_controller: MapsController { db: db.clone() },
        }
    }

    pub async fn start(&'static self) -> std::io::Result<()> {
        println!("Starting!");

        let app_env = env::var("APP_ENV").expect("APP_ENV must be set");

        if !["local", "staging", "prod"].contains(&app_env.as_str()) {
            panic!("APP_ENV must be one of local/staging/prod")
        }

        HttpServer::new(move || {
            let mut cors = Cors::default()
                .allowed_origin("https://www.planting.life")
                .allowed_origin("https://planting.life")
                .allowed_origin("https://maps.planting.life")
                .allowed_header(http::header::CONTENT_TYPE)
                .allowed_header(http::header::ACCEPT)
                .allowed_methods(vec!["GET", "POST", "PUT"]);

            // In non-prod environments, don't restrict origin.
            // This allows localhost, but also networked locations (ex: access
            // from phone on local network)
            if app_env != "prod" {
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
                .service(list_garden_handler)
                .service(create_garden_handler)
                .service(update_garden_handler)
                .service(maps_api_key_handler)
        })
        .bind("0.0.0.0:8080")?
        .run()
        .await
    }
}
