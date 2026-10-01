//! Command-line argument parsing and CLI interface definitions.
//!
//! Provides the [`Cli`] parser powered by `clap`, handling command-line flags,
//! positional arguments, subcommands, and environment variable overrides.
//!
//! This module is compiled exclusively when the non-default `cli` feature is enabled.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

/// Command-line configuration parser for the CoSave server process.
///
/// Gated behind the `cli` feature flag. Unifies command-line flags, options,
/// positional arguments, and subcommands to configure network binding, environment
/// mode, logging verbosity, and static asset hosting.
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
    /// Returns the requested environment mode string (`"DEV"` or `"PROD"`), if specified.
    ///
    /// Checks both the `--env` flag and positional `ENV` argument. Takes precedence over
    /// the `COSAVE_ENV` environment variable during configuration resolution.
    pub fn env(&self) -> Option<&str> {
        self.env.as_deref().or(self.env_pos.as_deref())
    }

    /// Returns the network host IP address or hostname to bind to, if specified.
    ///
    /// Controlled via `-H` or `--host`. Takes precedence over the `COSAVE_HOST`
    /// environment variable. Defaults to `0.0.0.0` when omitted.
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// Returns the TCP port to bind to, if specified.
    ///
    /// Controlled via `-p` or `--port`. Takes precedence over the `COSAVE_PORT`
    /// environment variable. When omitted, the default port is derived from the
    /// operating environment (5171 for `DEV`, 5172 for `PROD`).
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Returns the path to the custom directory containing static frontend SPA assets, if specified.
    ///
    /// Controlled via `--static-dir`. Takes precedence over the `COSAVE_STATIC_DIR`
    /// environment variable. When omitted, default distribution locations are used.
    pub fn static_dir(&self) -> Option<&Path> {
        self.static_dir.as_deref()
    }

    /// Returns whether static asset hosting is disabled, serving only `/api/v1` routes.
    ///
    /// Evaluates to `true` when passing the `--api` flag or invoking the `api` subcommand.
    pub fn api_only(&self) -> bool {
        self.api || matches!(self.command, Some(Commands::Api))
    }

    /// Returns whether verbose/debug logging output is requested.
    ///
    /// Controlled via `-v`, `--verbose`, or `--debug`.
    pub fn is_verbose(&self) -> bool {
        self.is_verbose
    }

    /// Parses command-line arguments from `std::env::args`.
    ///
    /// Inspects OS process arguments and constructs a validated [`Cli`] instance, printing
    /// standard help or version output and exiting if `--help` or `--version` is supplied.
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
        let cli = parse_from(vec!["--host", "127.0.0.1", "-p", "2300", "-v"]).unwrap();
        assert_eq!(cli.host(), Some("127.0.0.1"));
        assert_eq!(cli.port(), Some(2300));
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
