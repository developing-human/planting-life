use std::env;

use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use planting_life::{
    database::Database,
    highlights::Highlights,
    routes::{
        gardens::{
            create_garden_handler, list_garden_handler, read_garden_handler, update_garden_handler,
        },
        maps::maps_api_key_handler,
        nurseries::fetch_nurseries_handler,
        plants::{find_plant_handler, find_plants_handler},
    },
    state::AppState,
};
use tracing::log::warn;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_url = env::var("PLANTING_LIFE_DB_URL").unwrap_or_else(|_| {
        warn!("Configure valid PLANTING_LIFE_DB_URL to use database");
        "".to_string()
    });

    let app_env = env::var("APP_ENV").expect("APP_ENV must be set");
    if !["local", "staging", "prod"].contains(&app_env.as_str()) {
        panic!("APP_ENV must be one of local/staging/prod")
    }

    let state = web::Data::new(AppState {
        db: Database::new(&db_url),
        highlights: Highlights {},
    });

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
            .app_data(state.clone())
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
