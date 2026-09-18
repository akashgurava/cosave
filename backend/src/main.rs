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
use std::{
    env,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    path::PathBuf,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppEnv {
    Dev,
    Prod,
}

impl AppEnv {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.trim().to_uppercase().as_str() {
            "DEV" | "DEVELOPMENT" => Ok(Self::Dev),
            "PROD" | "PRODUCTION" => Ok(Self::Prod),
            other => Err(format!(
                "Invalid environment '{other}'. Expected 'DEV' or 'PROD'."
            )),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Dev => "DEV",
            Self::Prod => "PROD",
        }
    }

    fn default_port(&self) -> u16 {
        match self {
            Self::Dev => 5171,
            Self::Prod => 5172,
        }
    }
}

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

    let app_env = if let Some(ref env_str) = cli.env {
        AppEnv::from_str(env_str).unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            std::process::exit(1);
        })
    } else if let Ok(env_str) = env::var("COSAVE_ENV") {
        AppEnv::from_str(&env_str).unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            std::process::exit(1);
        })
    } else {
        AppEnv::Dev
    };

    tracing::info!("Environment resolved to: {}", app_env.as_str());

    let db_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/cosave.db?mode=rwc".to_string());
    let db = db::init_db(&db_url)
        .await
        .expect("Failed to initialize SQLite database");
    let state = AppState::new(db);

    let api_router = routes::router().with_state(state);

    let mut app = Router::new()
        .nest("/api/v1", api_router)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(
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
        let static_dir_opt = cli
            .static_dir
            .or_else(|| env::var("COSAVE_STATIC_DIR").ok().map(PathBuf::from));

        let static_path = match static_dir_opt {
            Some(path) => path,
            None => {
                eprintln!(
                    "Error: Static directory is required when not running in API-only mode.\n\
                     Provide --static-dir <PATH> or set COSAVE_STATIC_DIR, or run with 'api' for API-only mode."
                );
                std::process::exit(1);
            }
        };

        if !static_path.exists() {
            eprintln!(
                "Error: Configured static directory '{}' does not exist.",
                static_path.display()
            );
            std::process::exit(1);
        }

        tracing::info!("Serving static files from '{}'", static_path.display());
        let index_path = static_path.join("index.html");
        let serve_dir = ServeDir::new(&static_path).not_found_service(ServeFile::new(index_path));
        app = app.fallback_service(serve_dir);
    }

    let host_str = cli
        .host
        .or_else(|| env::var("COSAVE_HOST").ok())
        .unwrap_or_else(|| "0.0.0.0".to_string());

    let default_port = app_env.default_port();
    let port = cli
        .port
        .or_else(|| env::var("COSAVE_PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(default_port);

    let addr: SocketAddr = match host_str.parse::<IpAddr>() {
        Ok(ip) => SocketAddr::new(ip, port),
        Err(_) => match (host_str.as_str(), port).to_socket_addrs() {
            Ok(mut addrs) => match addrs.next() {
                Some(a) => a,
                None => {
                    eprintln!("Error: unable to resolve host '{host_str}' to a socket address");
                    std::process::exit(1);
                }
            },
            Err(err) => {
                eprintln!("Error: invalid host address '{host_str}': {err}");
                std::process::exit(1);
            }
        },
    };
    tracing::info!("CoSave server listening on http://{}", addr,);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_env_parsing() {
        assert_eq!(AppEnv::from_str("dev").unwrap(), AppEnv::Dev);
        assert_eq!(AppEnv::from_str("DEV").unwrap(), AppEnv::Dev);
        assert_eq!(AppEnv::from_str("development").unwrap(), AppEnv::Dev);
        assert_eq!(AppEnv::from_str("DEVELOPMENT").unwrap(), AppEnv::Dev);

        assert_eq!(AppEnv::from_str("prod").unwrap(), AppEnv::Prod);
        assert_eq!(AppEnv::from_str("PROD").unwrap(), AppEnv::Prod);
        assert_eq!(AppEnv::from_str("production").unwrap(), AppEnv::Prod);
        assert_eq!(AppEnv::from_str("PRODUCTION").unwrap(), AppEnv::Prod);

        assert!(AppEnv::from_str("staging").is_err());
        assert!(AppEnv::from_str("").is_err());
    }

    #[test]
    fn test_app_env_default_ports() {
        assert_eq!(AppEnv::Dev.default_port(), 5171);
        assert_eq!(AppEnv::Prod.default_port(), 5172);
    }

    #[test]
    fn test_app_env_as_str() {
        assert_eq!(AppEnv::Dev.as_str(), "DEV");
        assert_eq!(AppEnv::Prod.as_str(), "PROD");
    }
}
