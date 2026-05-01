use mockall_double::double;

#[double]
use crate::database::Database;
use crate::highlights::Highlights;
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<Database>,
    pub highlights: Arc<Highlights>,
}
