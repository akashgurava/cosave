use std::sync::Arc;

use crate::core::{AppConfig, DbPool};

/// Shared application state injected into Axum routes and extractors.
#[derive(Clone)]
pub struct AppState {
    db: DbPool,
    config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(db: DbPool, config: Arc<AppConfig>) -> Self {
        Self { db, config }
    }

    pub fn for_test(db: DbPool) -> Self {
        Self {
            db,
            config: Arc::new(AppConfig::for_test("sqlite::memory:")),
        }
    }

    pub fn db(&self) -> &DbPool {
        &self.db
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}
