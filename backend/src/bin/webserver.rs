use std::env;

use planting_life::app::PlantingLifeApp;
use tracing::log::warn;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_url = env::var("PLANTING_LIFE_DB_URL").unwrap_or_else(|_| {
        warn!("Configure valid PLANTING_LIFE_DB_URL to use database");
        "".to_string()
    });
    let app = PlantingLifeApp::new(&db_url);

    // Leak it to get a 'static lifetime, by definition it lives for
    // the entirety of the program
    let app = Box::leak(Box::new(app));

    app.start().await
}
