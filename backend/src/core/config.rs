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
    /// Strictly parses environment string ("DEV" or "PROD" only).
    pub fn from_str_strict(s: &str) -> Result<Self, String> {
        match s {
            "DEV" => Ok(Self::Dev),
            "PROD" => Ok(Self::Prod),
            other => Err(format!(
                "Invalid environment '{other}'. Expected 'DEV' or 'PROD'."
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dev => "DEV",
            Self::Prod => "PROD",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            Self::Dev => 5171,
            Self::Prod => 5172,
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
            host: "127.0.0.1".to_string(),
            port: 5171,
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

#[cfg(feature = "cli")]
impl AppConfig {
    /// Loads configuration by unifying CLI flags and environment variables with strict validation.
    pub fn from_cli_and_env(cli: &Cli) -> Result<Self, String> {
        let app_env = if let Some(env_str) = cli.env() {
            AppEnv::from_str_strict(env_str)?
        } else if let Ok(env_str) = env::var("COSAVE_ENV") {
            AppEnv::from_str_strict(&env_str)?
        } else {
            AppEnv::Dev
        };

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://data/cosave.db?mode=rwc".to_string());

        let host = cli
            .host()
            .map(ToString::to_string)
            .or_else(|| env::var("COSAVE_HOST").ok())
            .unwrap_or_else(|| "0.0.0.0".to_string());

        let default_port = app_env.default_port();
        let port = if let Some(p) = cli.port() {
            p
        } else if let Ok(port_str) = env::var("COSAVE_PORT") {
            port_str.parse::<u16>().map_err(|_| {
                format!(
                    "Invalid port '{port_str}' in COSAVE_PORT. Expected a 16-bit unsigned integer."
                )
            })?
        } else {
            default_port
        };

        let static_dir = cli
            .static_dir()
            .map(Path::to_path_buf)
            .or_else(|| env::var("COSAVE_STATIC_DIR").ok().map(PathBuf::from));

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
        let app_env = if let Ok(env_str) = env::var("COSAVE_ENV") {
            AppEnv::from_str_strict(&env_str)?
        } else {
            AppEnv::Dev
        };

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://data/cosave.db?mode=rwc".to_string());

        let host = env::var("COSAVE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let default_port = app_env.default_port();
        let port = if let Ok(port_str) = env::var("COSAVE_PORT") {
            port_str.parse::<u16>().map_err(|_| {
                format!(
                    "Invalid port '{port_str}' in COSAVE_PORT. Expected a 16-bit unsigned integer."
                )
            })?
        } else {
            default_port
        };

        let static_dir = env::var("COSAVE_STATIC_DIR").ok().map(PathBuf::from);

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
        let config = AppConfig::for_test("sqlite::memory:");
        assert_eq!(config.env(), AppEnv::Dev);
        assert_eq!(config.database_url(), "sqlite::memory:");
        assert_eq!(config.host(), "127.0.0.1");
        assert_eq!(config.port(), 5171);
        assert!(config.static_dir().is_none());
        assert!(config.api_only());
        assert!(!config.is_verbose());
    }
}
