//! Application startup configuration and environment management.
//!
//! # Architecture & Single Source of Truth (SSOT)
//! In accordance with CoSave architecture standards, environment variables and CLI inputs
//! are parsed and validated strictly once at application boot into an immutable [`AppConfig`].
//! Dynamic runtime environment lookups (`std::env::var`) across route handlers, domain logic,
//! or database layers are strictly prohibited.
//!
//! # Operating Modes
//! - **CLI Mode** (`feature = "cli"`): CLI arguments ([`Cli`]) take precedence over environment variables,
//!   which in turn fall back to authoritative system defaults.
//! - **Headless Mode** (`not(feature = "cli")`): Configuration is resolved strictly from environment
//!   variables and default values, ideal for minimal container deployments.

use std::env;
use std::path::{Path, PathBuf};

#[cfg(feature = "cli")]
use super::Cli;

/// Operating environment mode enforcing exact representation matching.
///
/// In CoSave, environment mode strings must match `"DEV"` or `"PROD"` exactly.
/// Loose case-insensitive matching or aliases (such as "development", "production", or "dev")
/// are rejected to guarantee deterministic behavior between local development and production.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    /// Development mode (default port 5171, verbose diagnostics).
    Dev,
    /// Production mode (default port 5172, optimized for containerized static SPA hosting).
    Prod,
}

impl AppEnv {
    /// Canonical string representation for development mode (`"DEV"`).
    pub const DEV_STR: &'static str = "DEV";

    /// Canonical string representation for production mode (`"PROD"`).
    pub const PROD_STR: &'static str = "PROD";

    /// Default TCP port assigned in development mode (`5171`).
    pub const DEFAULT_DEV_PORT: u16 = 5171;

    /// Default TCP port assigned in production mode (`5172`).
    pub const DEFAULT_PROD_PORT: u16 = 5172;

    /// Strictly parses an environment string into [`AppEnv`].
    ///
    /// # Errors
    /// Returns an error if the input string does not match [`Self::DEV_STR`] or [`Self::PROD_STR`] exactly.
    pub fn from_str_strict(s: &str) -> Result<Self, String> {
        if s == Self::DEV_STR {
            Ok(Self::Dev)
        } else if s == Self::PROD_STR {
            Ok(Self::Prod)
        } else {
            Err(format!(
                "Invalid environment '{s}'. Expected '{}' or '{}'.",
                Self::DEV_STR,
                Self::PROD_STR
            ))
        }
    }

    /// Returns the static string slice representation (`"DEV"` or `"PROD"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dev => Self::DEV_STR,
            Self::Prod => Self::PROD_STR,
        }
    }

    /// Returns the authoritative default port associated with this environment mode.
    pub fn default_port(&self) -> u16 {
        match self {
            Self::Dev => Self::DEFAULT_DEV_PORT,
            Self::Prod => Self::DEFAULT_PROD_PORT,
        }
    }
}

/// Immutable application runtime configuration parsed and validated once at startup.
///
/// Encapsulates network binding settings, database connection URLs, static asset serving options,
/// and logging verbosity. Fields are private and exposed via read-only accessors.
#[derive(Debug, Clone)]
pub struct AppConfig {
    env: AppEnv,
    database_url: String,
    host: String,
    port: u16,
    static_dir: Option<PathBuf>,
    api_only: bool,
    is_verbose: bool,
}

impl AppConfig {
    /// Authoritative environment variable name for environment mode (`"COSAVE_ENV"`).
    pub const ENV_VAR_ENV: &'static str = "COSAVE_ENV";

    /// Authoritative environment variable name for SQLite database URL (`"COSAVE_DATABASE_URL"`).
    pub const ENV_VAR_DATABASE_URL: &'static str = "COSAVE_DATABASE_URL";

    /// Authoritative environment variable name for bind host (`"COSAVE_HOST"`).
    pub const ENV_VAR_HOST: &'static str = "COSAVE_HOST";

    /// Authoritative environment variable name for bind port (`"COSAVE_PORT"`).
    pub const ENV_VAR_PORT: &'static str = "COSAVE_PORT";

    /// Authoritative environment variable name for static SPA directory (`"COSAVE_STATIC_DIR"`).
    pub const ENV_VAR_STATIC_DIR: &'static str = "COSAVE_STATIC_DIR";

    /// Default network host binding (`"0.0.0.0"`).
    pub const DEFAULT_HOST: &'static str = "0.0.0.0";

    /// Default network host binding for test harnesses (`"127.0.0.1"`).
    pub const DEFAULT_TEST_HOST: &'static str = "127.0.0.1";

    /// Default SQLite database URL (`"sqlite://data/cosave.db?mode=rwc"`).
    pub const DEFAULT_DATABASE_URL: &'static str = "sqlite://data/cosave.db?mode=rwc";

    /// Standard in-memory SQLite database URL used for test suites (`"sqlite::memory:"`).
    pub const IN_MEMORY_DATABASE_URL: &'static str = "sqlite::memory:";

    /// Constructs an explicit [`AppConfig`] instance.
    #[must_use]
    pub fn new(
        env: AppEnv,
        database_url: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        static_dir: Option<PathBuf>,
        api_only: bool,
        is_verbose: bool,
    ) -> Self {
        Self {
            env,
            database_url: database_url.into(),
            host: host.into(),
            port,
            static_dir,
            api_only,
            is_verbose,
        }
    }

    /// Constructs an [`AppConfig`] tailored for test harnesses using an in-memory database.
    ///
    /// Defaults to [`AppEnv::Dev`], [`Self::DEFAULT_TEST_HOST`], [`AppEnv::DEFAULT_DEV_PORT`],
    /// API-only mode, and quiet logging.
    pub fn for_test(database_url: impl Into<String>) -> Self {
        Self {
            env: AppEnv::Dev,
            database_url: database_url.into(),
            host: Self::DEFAULT_TEST_HOST.to_string(),
            port: AppEnv::DEFAULT_DEV_PORT,
            static_dir: None,
            api_only: true,
            is_verbose: false,
        }
    }

    /// Returns the operating environment mode ([`AppEnv::Dev`] or [`AppEnv::Prod`]).
    pub fn env(&self) -> AppEnv {
        self.env
    }

    /// Returns the SQLite connection string / database URL.
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Returns the network host IP address or hostname to bind.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the TCP port to bind.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the custom directory containing static SPA frontend assets, if specified.
    pub fn static_dir(&self) -> Option<&Path> {
        self.static_dir.as_deref()
    }

    /// Returns whether static asset hosting is disabled, serving only `/api/v1` routes.
    pub fn api_only(&self) -> bool {
        self.api_only
    }

    /// Returns whether verbose/debug logging is enabled.
    pub fn is_verbose(&self) -> bool {
        self.is_verbose
    }
}

fn resolve_env_from_var() -> Result<Option<AppEnv>, String> {
    if let Ok(env_str) = env::var(AppConfig::ENV_VAR_ENV) {
        AppEnv::from_str_strict(&env_str).map(Some)
    } else {
        Ok(None)
    }
}

fn resolve_database_url() -> String {
    env::var(AppConfig::ENV_VAR_DATABASE_URL)
        .unwrap_or_else(|_| AppConfig::DEFAULT_DATABASE_URL.to_string())
}

fn resolve_env_host() -> Option<String> {
    env::var(AppConfig::ENV_VAR_HOST).ok()
}

fn resolve_env_static_dir() -> Option<PathBuf> {
    env::var(AppConfig::ENV_VAR_STATIC_DIR)
        .ok()
        .map(PathBuf::from)
}

fn parse_port(port_str: &str) -> Result<u16, String> {
    port_str.parse::<u16>().map_err(|_| {
        format!(
            "Invalid port '{port_str}' in {}. Expected a 16-bit unsigned integer.",
            AppConfig::ENV_VAR_PORT
        )
    })
}

#[cfg(feature = "cli")]
impl AppConfig {
    /// Loads and validates configuration by unifying CLI flags, environment variables, and defaults.
    ///
    /// # Resolution Precedence
    /// 1. **Explicit CLI flags** passed to [`Cli`] (`--host`, `--port`, `--env`, `--static-dir`, etc.)
    /// 2. **Authoritative environment variables** (`COSAVE_HOST`, `COSAVE_PORT`, `COSAVE_ENV`, etc.)
    /// 3. **Static defaults** defined on [`AppConfig`] and [`AppEnv`].
    ///
    /// # Errors
    /// Returns an error message if the environment mode string is invalid (must be `"DEV"` or `"PROD"`),
    /// or if the port string cannot be parsed as a 16-bit unsigned integer.
    pub fn from_cli_and_env(cli: &Cli) -> Result<Self, String> {
        let app_env = if let Some(env_str) = cli.env() {
            AppEnv::from_str_strict(env_str)?
        } else {
            resolve_env_from_var()?.unwrap_or(AppEnv::Dev)
        };

        let database_url = resolve_database_url();

        let host = cli
            .host()
            .map(ToString::to_string)
            .or_else(resolve_env_host)
            .unwrap_or_else(|| Self::DEFAULT_HOST.to_string());

        let default_port = app_env.default_port();
        let port = if let Some(p) = cli.port() {
            p
        } else if let Ok(port_str) = env::var(Self::ENV_VAR_PORT) {
            parse_port(&port_str)?
        } else {
            default_port
        };

        let static_dir = cli
            .static_dir()
            .map(Path::to_path_buf)
            .or_else(resolve_env_static_dir);

        let api_only = cli.api_only();
        let is_verbose = cli.is_verbose();

        Ok(Self {
            env: app_env,
            database_url,
            host,
            port,
            static_dir,
            api_only,
            is_verbose,
        })
    }
}

#[cfg(not(feature = "cli"))]
impl AppConfig {
    /// Loads and validates configuration exclusively from environment variables and system defaults.
    ///
    /// Used when the server crate is compiled without the `cli` feature (e.g. headless container images).
    ///
    /// # Resolution Precedence
    /// 1. **Authoritative environment variables** (`COSAVE_HOST`, `COSAVE_PORT`, `COSAVE_ENV`, etc.)
    /// 2. **Static defaults** defined on [`AppConfig`] and [`AppEnv`].
    ///
    /// # Errors
    /// Returns an error message if the environment mode string is invalid (must be `"DEV"` or `"PROD"`),
    /// or if the port string cannot be parsed as a 16-bit unsigned integer.
    pub fn load_from_env() -> Result<Self, String> {
        let app_env = resolve_env_from_var()?.unwrap_or(AppEnv::Dev);
        let database_url = resolve_database_url();
        let host = resolve_env_host().unwrap_or_else(|| Self::DEFAULT_HOST.to_string());

        let default_port = app_env.default_port();
        let port = if let Ok(port_str) = env::var(Self::ENV_VAR_PORT) {
            parse_port(&port_str)?
        } else {
            default_port
        };

        let static_dir = resolve_env_static_dir();

        Ok(Self {
            env: app_env,
            database_url,
            host,
            port,
            static_dir,
            api_only: true,
            is_verbose: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_env_strict_parsing() {
        assert_eq!(AppEnv::from_str_strict("DEV").unwrap(), AppEnv::Dev);
        assert_eq!(AppEnv::from_str_strict("PROD").unwrap(), AppEnv::Prod);
        assert!(AppEnv::from_str_strict("dev").is_err());
        assert!(AppEnv::from_str_strict("prod").is_err());
        assert!(AppEnv::from_str_strict("development").is_err());
        assert!(AppEnv::from_str_strict("").is_err());
    }

    #[test]
    fn test_app_env_properties() {
        assert_eq!(AppEnv::Dev.as_str(), "DEV");
        assert_eq!(AppEnv::Prod.as_str(), "PROD");
        assert_eq!(AppEnv::Dev.default_port(), 5171);
        assert_eq!(AppEnv::Prod.default_port(), 5172);
    }

    #[test]
    fn test_for_test_configuration() {
        let config = AppConfig::for_test(AppConfig::IN_MEMORY_DATABASE_URL);
        assert_eq!(config.env(), AppEnv::Dev);
        assert_eq!(config.database_url(), AppConfig::IN_MEMORY_DATABASE_URL);
        assert_eq!(config.host(), AppConfig::DEFAULT_TEST_HOST);
        assert_eq!(config.port(), AppEnv::DEFAULT_DEV_PORT);
        assert!(config.static_dir().is_none());
        assert!(config.api_only());
        assert!(!config.is_verbose());
    }

    #[test]
    fn test_parse_port() {
        assert_eq!(parse_port("8080").unwrap(), 8080);
        assert!(parse_port("invalid").is_err());
        assert!(parse_port("-1").is_err());
        assert!(parse_port("70000").is_err());
    }

    #[test]
    fn test_app_config_new() {
        let config = AppConfig::new(
            AppEnv::Prod,
            "sqlite://custom.db",
            "127.0.0.1",
            8080,
            Some(PathBuf::from("/static")),
            false,
            true,
        );
        assert_eq!(config.env(), AppEnv::Prod);
        assert_eq!(config.database_url(), "sqlite://custom.db");
        assert_eq!(config.host(), "127.0.0.1");
        assert_eq!(config.port(), 8080);
        assert_eq!(config.static_dir(), Some(Path::new("/static")));
        assert!(!config.api_only());
        assert!(config.is_verbose());
    }
}
