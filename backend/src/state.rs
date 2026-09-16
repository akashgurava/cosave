use crate::db::DbPool;

/// Shared application state injected into Axum routes and extractors.
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: DbPool,
}

impl AppState {
    pub(crate) fn new(db: DbPool) -> Self {
        Self { db }
    }
}
