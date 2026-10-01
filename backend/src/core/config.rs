use std::env;
use std::path::{Path, PathBuf};

#[cfg(feature = "cli")]
use super::Cli;

/// Environment operating modes enforcing exact representation matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Dev,
    Prod,
}

impl AppEnv {
    pub const DEV_STR: &'static str = "DEV";
    pub const PROD_STR: &'static str = "PROD";
    pub const DEFAULT_DEV_PORT: u16 = 5171;
    pub const DEFAULT_PROD_PORT: u16 = 5172;

    /// Strictly parses environment string ("DEV" or "PROD" only).
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

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dev => Self::DEV_STR,
            Self::Prod => Self::PROD_STR,
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            Self::Dev => Self::DEFAULT_DEV_PORT,
            Self::Prod => Self::DEFAULT_PROD_PORT,
        }
    }
}

/// Immutable application runtime configuration parsed and validated once at startup.
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
    /// Authoritative environment variable names.
    pub const ENV_VAR_ENV: &'static str = "COSAVE_ENV";
    pub const ENV_VAR_DATABASE_URL: &'static str = "COSAVE_DATABASE_URL";
    pub const ENV_VAR_HOST: &'static str = "COSAVE_HOST";
    pub const ENV_VAR_PORT: &'static str = "COSAVE_PORT";
    pub const ENV_VAR_STATIC_DIR: &'static str = "COSAVE_STATIC_DIR";

    /// Authoritative default network and storage configurations.
    pub const DEFAULT_HOST: &'static str = "0.0.0.0";
    pub const DEFAULT_TEST_HOST: &'static str = "127.0.0.1";
    pub const DEFAULT_DATABASE_URL: &'static str = "sqlite://data/cosave.db?mode=rwc";
    pub const IN_MEMORY_DATABASE_URL: &'static str = "sqlite::memory:";

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

    /// Configuration for test harnesses using an in-memory SQLite database.
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

    pub fn env(&self) -> AppEnv {
        self.env
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn static_dir(&self) -> Option<&Path> {
        self.static_dir.as_deref()
    }

    pub fn api_only(&self) -> bool {
        self.api_only
    }

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
    /// Loads configuration by unifying CLI flags and environment variables with strict validation.
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
    /// Loads configuration exclusively from environment variables when built without CLI.
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
