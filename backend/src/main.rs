//! Application server entrypoint and binary bootstrap.
//!
//! This binary boots the backend by parsing command-line options and environment configuration,
//! initializing logging subscribers, and connecting to the SQLite database. It runs startup
//! schema migrations and default data seeding before launching the Axum web server. Depending on
//! configuration, the server runs in API-only mode or serves compiled frontend assets alongside the REST API,
//! with graceful termination on standard shutdown signals.

#![deny(dead_code)]

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cosave::{
    api_only_root_fallback, init_db, init_features, init_schemas, router, AppConfig, AppState, Cli,
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let default_filter = if cli.is_verbose() {
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

    let config = match AppConfig::from_cli_and_env(&cli) {
        Ok(c) => Arc::new(c),
        Err(err) => {
            tracing::error!("APP.BOOTSTRAP.CONFIG_PARSE_FAILED. Invalid configuration: {err}");
            std::process::exit(1);
        }
    };

    tracing::info!(
        "APP.BOOTSTRAP.ENV_RESOLVED. Environment resolved to: {}",
        config.env().as_str()
    );

    let db = match init_db(config.database_url()).await {
        Ok(pool) => pool,
        Err(err) => {
            tracing::error!(
                error = %err,
                action = err.action(),
                code = err.code(),
                "APP.BOOTSTRAP.INIT_DB_FAILED. Failed to initialize database"
            );
            std::process::exit(1);
        }
    };

    if let Err(err) = init_schemas(&db).await {
        tracing::error!(
            error = %err,
            action = err.action(),
            code = err.code(),
            "APP.BOOTSTRAP.INIT_SCHEMAS_FAILED. Failed to run schema migrations"
        );
        std::process::exit(1);
    }

    if let Err(err) = init_features(&db).await {
        tracing::error!(
            error = %err,
            action = err.action(),
            code = err.code(),
            "APP.BOOTSTRAP.INIT_FEATURES_FAILED. Failed to initialize feature modules"
        );
        std::process::exit(1);
    }

    let state = AppState::new(db, Arc::clone(&config));

    let api_router = router().with_state(state);

    let mut app = Router::new()
        .nest("/api/v1", api_router)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    if config.api_only() {
        tracing::info!(
            "APP.BOOTSTRAP.API_MODE. Running in API-only mode (static file serving disabled)"
        );
        app = app.fallback(api_only_root_fallback);
    } else {
        let static_path = match config.static_dir() {
            Some(path) => path,
            None => {
                tracing::error!(
                    "APP.BOOTSTRAP.STATIC_DIR_REQUIRED. Static directory is required when not running in API-only mode. Provide --static-dir <PATH> or set {}, or run with 'api' for API-only mode.",
                    AppConfig::ENV_VAR_STATIC_DIR
                );
                std::process::exit(1);
            }
        };

        if !static_path.exists() {
            tracing::error!(
                path = %static_path.display(),
                "APP.BOOTSTRAP.STATIC_DIR_NOT_FOUND. Configured static directory does not exist"
            );
            std::process::exit(1);
        }

        let index_path = static_path.join("index.html");
        if !index_path.exists() {
            tracing::error!(
                path = %index_path.display(),
                "APP.BOOTSTRAP.INDEX_HTML_NOT_FOUND. index.html was not found in static directory"
            );
            std::process::exit(1);
        }

        tracing::info!(
            "APP.BOOTSTRAP.STATIC_FILES. Serving static files from '{}'",
            static_path.display()
        );
        let serve_dir = ServeDir::new(static_path).not_found_service(ServeFile::new(index_path));
        app = app.fallback_service(serve_dir);
    }

    let host_str = config.host();
    let port = config.port();

    let addr: SocketAddr = match host_str.parse::<IpAddr>() {
        Ok(ip) => SocketAddr::new(ip, port),
        Err(_) => match (host_str, port).to_socket_addrs() {
            Ok(mut addrs) => match addrs.next() {
                Some(a) => a,
                None => {
                    tracing::error!(
                        host = %host_str,
                        "APP.BOOTSTRAP.HOST_RESOLUTION_EMPTY. Unable to resolve host to a socket address"
                    );
                    std::process::exit(1);
                }
            },
            Err(err) => {
                tracing::error!(
                    host = %host_str,
                    error = %err,
                    "APP.BOOTSTRAP.INVALID_HOST. Invalid host address"
                );
                std::process::exit(1);
            }
        },
    };

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(err) => {
            tracing::error!(
                error = %err,
                addr = %addr,
                "APP.STARTUP.BIND_FAILED. Failed to bind TCP listener"
            );
            std::process::exit(1);
        }
    };

    tracing::info!(
        "APP.STARTUP.LISTENING. CoSave server listening on http://{}",
        addr
    );

    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!(
            error = %err,
            "APP.STARTUP.SERVE_FAILED. Server error encountered"
        );
        std::process::exit(1);
    }
}

/// Waits for a SIGINT (Ctrl+C) or SIGTERM signal to trigger graceful server shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {}
            Err(err) => {
                tracing::error!(
                    error = %err,
                    "APP.SHUTDOWN.CTRL_C_INSTALL_FAILED. Failed to install Ctrl+C handler"
                );
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(err) => {
                tracing::error!(
                    error = %err,
                    "APP.SHUTDOWN.SIGTERM_INSTALL_FAILED. Failed to install SIGTERM handler"
                );
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("APP.SHUTDOWN.SIGNAL_CTRL_C. Received Ctrl+C, initiating graceful shutdown")
        }
        _ = terminate => {
            tracing::info!("APP.SHUTDOWN.SIGNAL_SIGTERM. Received SIGTERM, initiating graceful shutdown")
        }
    }
}
