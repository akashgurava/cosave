mod auth;
mod cli;
mod db;
mod models;
mod response;
mod routes;
mod state;

use axum::{http::StatusCode, Router};
use cli::Cli;
use state::AppState;
use std::{env, net::SocketAddr, path::PathBuf};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let cli = match Cli::parse() {
        Ok(cli) => cli,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(if msg.contains("Usage:") { 0 } else { 1 });
        }
    };

    let default_filter = if cli.is_verbose {
        "cosave=debug,tower_http=debug"
    } else {
        "cosave=info,tower_http=info"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::debug!("Debug logging enabled via verbose flag");

    let db_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/cosave.db?mode=rwc".to_string());
    let db = db::init_db(&db_url)
        .await
        .expect("Failed to initialize SQLite database");
    let state = AppState::new(db);

    let api_router = routes::router().with_state(state);

    let mut app = Router::new().nest("/api/v1", api_router).layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    if cli.api_only {
        tracing::info!("Running in API-only mode (static file serving disabled)");
        app = app.fallback(|| async {
            (
                StatusCode::NOT_FOUND,
                "API endpoint not found. Note: Server is running in API-only mode.",
            )
        });
    } else {
        let (static_path, is_explicit_static_dir) = if let Some(dir) = cli.static_dir {
            (dir, true)
        } else if let Ok(dir) = env::var("STATIC_DIR") {
            (PathBuf::from(dir), true)
        } else {
            let candidates = ["./dist", "./frontend/dist", "../frontend/dist"];
            let found = candidates
                .iter()
                .map(PathBuf::from)
                .find(|p| p.exists())
                .unwrap_or_else(|| PathBuf::from("./dist"));
            (found, false)
        };

        let index_path = static_path.join("index.html");

        if static_path.exists() {
            tracing::info!("Serving static files from '{}'", static_path.display());
            let serve_dir =
                ServeDir::new(&static_path).not_found_service(ServeFile::new(index_path));
            app = app.fallback_service(serve_dir);
        } else {
            if is_explicit_static_dir {
                tracing::warn!(
                    "Configured static directory '{}' not found. Static file serving will return 404.",
                    static_path.display()
                );
            } else {
                tracing::debug!(
                    "No prebuilt static directory found (running in API mode; UI is served via Vite)."
                );
            }
            app = app.fallback(|| async {
                (
                    StatusCode::NOT_FOUND,
                    "Static frontend files not found. In development, access the UI via the Vite dev server (http://localhost:5173).",
                )
            });
        }
    }

    let port = cli
        .port
        .or_else(|| env::var("PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("CoSave server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

/// Waits for a SIGINT (Ctrl+C) or SIGTERM signal to trigger graceful server shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received Ctrl+C, initiating graceful shutdown"),
        _ = terminate => tracing::info!("Received SIGTERM, initiating graceful shutdown"),
    }
}
