mod application;
mod domain;
mod infrastructure;

use std::sync::Arc;

pub use infrastructure::db;
pub use infrastructure::persistence::postgres;

#[derive(Clone)]
pub struct AppContext {
    pub party_repo: Arc<postgres::SqlxPartyRepository>,
}

pub use infrastructure::transport;
