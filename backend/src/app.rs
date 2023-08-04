use crate::{
    controllers::{
        nurseries::{fetch_nurseries_handler, NurseriesController},
        plants::{fetch_plants_handler, PlantController},
    },
    database::MariaDB,
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

        let db = MariaDB::new(db_url);
        let db = Box::leak(Box::new(db));
        let plant_controller = PlantController::new(db);
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
