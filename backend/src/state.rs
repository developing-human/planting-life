use mockall_double::double;

#[double]
use crate::database::Database;
use crate::highlights::Highlights;

pub struct AppState {
    pub db: Database,
    pub highlights: Highlights,
}
