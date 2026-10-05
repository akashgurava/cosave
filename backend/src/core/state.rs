//! Shared application runtime state and extractor encapsulation.
//!
//! Encapsulates the shared SQLite connection pool and immutable configuration within
//! the thread-safe [`AppState`] container. This state is injected into the root Axum router
//! at startup and extracted by route handlers to interact with the database.
//! Keeping internal fields private ensures database handles and runtime settings are
//! accessed consistently across all feature modules.

use std::sync::Arc;

use super::{AppConfig, DbPool};

/// Thread-safe shared application runtime state injected into Axum routes and extractors.
///
/// Encapsulates the SQLite connection pool ([`DbPool`]) and immutable startup configuration
/// ([`AppConfig`]). Fields are strictly private to guarantee encapsulation, accessible
/// only via reference getters ([`Self::db`] and [`Self::config`]).
#[derive(Clone)]
pub struct AppState {
    db: DbPool,
    config: Arc<AppConfig>,
}

impl AppState {
    /// Constructs a new [`AppState`] with the provided database pool and configuration.
    pub fn new(db: DbPool, config: Arc<AppConfig>) -> Self {
        Self { db, config }
    }

    /// Constructs an [`AppState`] configured for test suites using test defaults and in-memory database configuration.
    pub fn for_test(db: DbPool) -> Self {
        Self {
            db,
            config: Arc::new(AppConfig::for_test(AppConfig::IN_MEMORY_DATABASE_URL)),
        }
    }

    /// Returns a borrowed reference to the shared SQLite connection pool.
    pub fn db(&self) -> &DbPool {
        &self.db
    }

    /// Returns a borrowed reference to the immutable application configuration.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}
