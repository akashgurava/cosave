use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

/// Command-line configuration for the server process.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "cosave",
    version,
    about = "CoSave — High-Performance Family Finance Server",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Environment mode positional argument (DEV or PROD).
    #[arg(value_name = "ENV")]
    env_pos: Option<String>,

    /// Environment mode (overrides COSAVE_ENV env var, defaults to DEV).
    #[arg(short = 'e', long = "env", global = true)]
    env: Option<String>,

    /// Host to listen on (overrides COSAVE_HOST env var, defaults to 0.0.0.0).
    #[arg(short = 'H', long = "host", global = true)]
    host: Option<String>,

    /// Port to listen on (overrides COSAVE_PORT env var; default calculated from env).
    /// PROD env -> 5172. DEV env -> 5171.
    #[arg(short = 'p', long = "port", global = true)]
    port: Option<u16>,

    /// Custom directory containing static SPA assets (overrides COSAVE_STATIC_DIR).
    #[arg(long = "static-dir", global = true)]
    static_dir: Option<PathBuf>,

    /// Disable static asset hosting and only serve `/api/v1` routes.
    #[arg(long = "api", global = true)]
    api: bool,

    /// Turn on verbose/debug logging output.
    #[arg(short = 'v', long = "verbose", alias = "debug", global = true)]
    is_verbose: bool,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
enum Commands {
    /// Run in API-only mode (disables static file requirement and serving)
    Api,
}

impl Cli {
    /// Environment mode string if specified.
    pub fn env(&self) -> Option<&str> {
        self.env.as_deref().or(self.env_pos.as_deref())
    }

    /// Host string to listen on if specified.
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// Port to listen on if specified.
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Custom directory path for static assets if specified.
    pub fn static_dir(&self) -> Option<&Path> {
        self.static_dir.as_deref()
    }

    /// Whether static asset hosting is disabled.
    pub fn api_only(&self) -> bool {
        self.api || matches!(self.command, Some(Commands::Api))
    }

    /// Whether debug/verbose logging output is requested.
    pub fn is_verbose(&self) -> bool {
        self.is_verbose
    }

    /// Parses CLI flags from `std::env::args`.
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_from<I, T>(args: I) -> Result<Cli, String>
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let mut full_args = vec!["cosave".to_string()];
        full_args.extend(
            args.into_iter()
                .map(Into::into)
                .filter(|a| !a.trim().is_empty()),
        );
        Cli::try_parse_from(full_args).map_err(|e| e.to_string())
    }

    #[test]
    fn test_default_cli_args() {
        let cli = parse_from(Vec::<String>::new()).unwrap();
        assert!(!cli.api_only());
        assert!(cli.static_dir().is_none());
        assert!(!cli.is_verbose());
        assert!(cli.host().is_none());
        assert!(cli.port().is_none());
        assert!(cli.env().is_none());
    }

    #[test]
    fn test_env_flag() {
        let cli = parse_from(vec!["-e", "DEV"]).unwrap();
        assert_eq!(cli.env(), Some("DEV"));

        let cli2 = parse_from(vec!["--env", "PROD"]).unwrap();
        assert_eq!(cli2.env(), Some("PROD"));

        let cli3 = parse_from(vec!["--env=development"]).unwrap();
        assert_eq!(cli3.env(), Some("development"));
    }

    #[test]
    fn test_api_subcommand() {
        let cli = parse_from(vec!["api"]).unwrap();
        assert!(cli.api_only());
    }

    #[test]
    fn test_static_dir_override() {
        let cli = parse_from(vec!["--static-dir", "./custom-dist"]).unwrap();
        assert_eq!(cli.static_dir(), Some(Path::new("./custom-dist")));
        assert!(!cli.api_only());
    }

    #[test]
    fn test_static_dir_equals_syntax() {
        let cli = parse_from(vec!["--static-dir=./abc"]).unwrap();
        assert_eq!(cli.static_dir(), Some(Path::new("./abc")));
    }

    #[test]
    fn test_verbose_flag() {
        let cli = parse_from(vec!["-v"]).unwrap();
        assert!(cli.is_verbose());

        let cli2 = parse_from(vec!["--verbose"]).unwrap();
        assert!(cli2.is_verbose());
    }

    #[test]
    fn test_host_flag() {
        let cli = parse_from(vec!["-H", "127.0.0.1"]).unwrap();
        assert_eq!(cli.host(), Some("127.0.0.1"));

        let cli2 = parse_from(vec!["--host", "0.0.0.0"]).unwrap();
        assert_eq!(cli2.host(), Some("0.0.0.0"));

        let cli3 = parse_from(vec!["--host=localhost"]).unwrap();
        assert_eq!(cli3.host(), Some("localhost"));
    }

    #[test]
    fn test_port_flag() {
        let cli = parse_from(vec!["--port", "4000"]).unwrap();
        assert_eq!(cli.port(), Some(4000));
    }

    #[test]
    fn test_combined_host_port_verbose() {
        let cli = parse_from(vec!["--host", "127.0.0.1", "-p", "5171", "-v"]).unwrap();
        assert_eq!(cli.host(), Some("127.0.0.1"));
        assert_eq!(cli.port(), Some(5171));
        assert!(cli.is_verbose());
    }

    #[test]
    fn test_combined_api_and_verbose() {
        let cli = parse_from(vec!["api", "-v"]).unwrap();
        assert!(cli.api_only());
        assert!(cli.is_verbose());
    }

    #[test]
    fn test_positional_env() {
        let cli = parse_from(vec!["DEV"]).unwrap();
        assert_eq!(cli.env(), Some("DEV"));

        let cli2 = parse_from(vec!["prod", "api"]).unwrap();
        assert_eq!(cli2.env(), Some("prod"));
        assert!(cli2.api_only());
    }

    #[test]
    fn test_api_flag() {
        let cli = parse_from(vec!["--api"]).unwrap();
        assert!(cli.api_only());
    }

    #[test]
    fn test_empty_argument_ignored() {
        let cli = parse_from(vec!["", "   ", "-v"]).unwrap();
        assert!(cli.is_verbose());
    }
}
