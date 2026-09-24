use super::db::DbPool;

/// Shared application state injected into Axum routes and extractors.
#[derive(Clone)]
pub struct AppState {
    db: DbPool,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbPool {
        &self.db
    }
}
