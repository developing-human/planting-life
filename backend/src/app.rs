use std::env;

use crate::{
    ai::{openai::OpenAI, RealAi},
    controllers::{
        nurseries::{fetch_nurseries_handler, NurseriesController},
        plants::{fetch_plants_handler, PlantController},
    },
    database::MariaDB,
    hydrator::RealHydrator,
};
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

pub struct PlantingLifeApp {
    pub plant_controller: PlantController,
    pub nursery_controller: NurseriesController,
}

impl PlantingLifeApp {
    pub fn new(db_url: &str) -> Self {
        tracing_subscriber::fmt::init();

        let db = live_forever(MariaDB::new(db_url));

        let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
        let open_ai = OpenAI::new(api_key);
        let ai = RealAi::new(open_ai);

        let hydrator = RealHydrator::new(Box::new(ai));

        let plant_controller = PlantController::new(db, Box::new(hydrator));
        let nursery_controller = NurseriesController::new(db);

        Self {
            plant_controller,
            nursery_controller,
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
