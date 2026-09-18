use std::{env, path::PathBuf};

/// Command-line configuration for the server process.
pub(crate) struct Cli {
    /// Environment mode (overrides COSAVE_ENV env var, defaults to DEV).
    pub(crate) env: Option<String>,
    /// Host to listen on (overrides COSAVE_HOST env var, defaults to 0.0.0.0).
    pub(crate) host: Option<String>,
    /// Port to listen on (overrides COSAVE_PORT env var; default calculated from env).
    /// PROD env -> 5172. DEV env -> 5171.
    pub(crate) port: Option<u16>,
    /// Custom directory containing static SPA assets (overrides COSAVE_STATIC_DIR).
    pub(crate) static_dir: Option<PathBuf>,
    /// Disable static asset hosting and only serve `/api/v1` routes.
    pub(crate) api_only: bool,
    /// Turn on verbose/debug logging output.
    pub(crate) is_verbose: bool,
}

impl Cli {
    /// Parses CLI flags from `std::env::args`.
    pub(crate) fn parse() -> Result<Self, String> {
        Self::parse_from(env::args().skip(1))
    }

    /// Parses an arbitrary iterator of argument strings (useful for unit testing).
    pub(crate) fn parse_from<I, T>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let mut env_mode = None;
        let mut host = None;
        let mut port = None;
        let mut static_dir = None;
        let mut api_only = false;
        let mut is_verbose = false;

        let args: Vec<String> = args.into_iter().map(Into::into).collect();
        let mut iter = args.into_iter().peekable();

        while let Some(arg) = iter.next() {
            if arg.trim().is_empty() {
                continue;
            }
            match arg.as_str() {
                "api" | "--api" => {
                    api_only = true;
                }
                "-v" | "--verbose" | "--debug" => {
                    is_verbose = true;
                }
                "-e" | "--env" => {
                    let val = iter.next().ok_or_else(|| {
                        "Error: '--env' requires an environment argument (DEV or PROD)".to_string()
                    })?;
                    if val.trim().is_empty() {
                        return Err("Error: '--env' cannot be empty".to_string());
                    }
                    env_mode = Some(val);
                }
                arg if arg.starts_with("--env=") => {
                    let val = arg.trim_start_matches("--env=");
                    if val.trim().is_empty() {
                        return Err("Error: '--env=' cannot be empty".to_string());
                    }
                    env_mode = Some(val.to_string());
                }
                "-H" | "--host" => {
                    let val = iter
                        .next()
                        .ok_or_else(|| "Error: '--host' requires a host address".to_string())?;
                    if val.trim().is_empty() {
                        return Err("Error: '--host' cannot be empty".to_string());
                    }
                    host = Some(val);
                }
                arg if arg.starts_with("--host=") => {
                    let val = arg.trim_start_matches("--host=");
                    if val.trim().is_empty() {
                        return Err("Error: '--host=' cannot be empty".to_string());
                    }
                    host = Some(val.to_string());
                }
                "-p" | "--port" => {
                    let val = iter
                        .next()
                        .ok_or_else(|| "Error: '--port' requires a port number".to_string())?;
                    let p = val
                        .parse::<u16>()
                        .map_err(|_| format!("Error: invalid port '{val}'"))?;
                    port = Some(p);
                }
                arg if arg.starts_with("--port=") => {
                    let val = arg.trim_start_matches("--port=");
                    let p = val
                        .parse::<u16>()
                        .map_err(|_| format!("Error: invalid port '{val}'"))?;
                    port = Some(p);
                }
                "--static-dir" => {
                    let val = iter.next().ok_or_else(|| {
                        "Error: '--static-dir' requires a directory path argument".to_string()
                    })?;
                    static_dir = Some(PathBuf::from(val));
                }
                arg if arg.starts_with("--static-dir=") => {
                    let val = arg.trim_start_matches("--static-dir=");
                    if val.is_empty() {
                        return Err("Error: '--static-dir=' cannot be empty".to_string());
                    }
                    static_dir = Some(PathBuf::from(val));
                }
                pos if env_mode.is_none()
                    && (pos.eq_ignore_ascii_case("DEV")
                        || pos.eq_ignore_ascii_case("PROD")
                        || pos.eq_ignore_ascii_case("DEVELOPMENT")
                        || pos.eq_ignore_ascii_case("PRODUCTION")) =>
                {
                    env_mode = Some(pos.to_string());
                }
                "-h" | "--help" => {
                    return Err(Self::help_text());
                }
                unknown => {
                    return Err(format!(
                        "Error: unknown argument '{unknown}'\n\n{}",
                        Self::help_text()
                    ));
                }
            }
        }

        Ok(Self {
            env: env_mode,
            host,
            port,
            static_dir,
            api_only,
            is_verbose,
        })
    }

    fn help_text() -> String {
        r#"CoSave — High-Performance Family Finance Server

Usage:
  cosave [ENV] [COMMAND] [OPTIONS]

Arguments:
  [ENV]                  Environment mode (DEV or PROD; default: DEV)

Commands:
  api                    Run in API-only mode (disables static file requirement and serving)

Options:
  -e, --env <ENV>        Specify environment (DEV or PROD; overrides COSAVE_ENV, default: DEV)
  -H, --host <HOST>      Specify host to listen on (overrides COSAVE_HOST, default: 0.0.0.0)
  -p, --port <PORT>      Specify port to listen on (overrides COSAVE_PORT and env default)
  --static-dir <PATH>    Directory for serving frontend static files (overrides COSAVE_STATIC_DIR)
  -v, --verbose, --debug Enable debug level logging
  -h, --help             Print help information
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cli_args() {
        let cli = Cli::parse_from(Vec::<String>::new()).unwrap();
        assert!(!cli.api_only);
        assert!(cli.static_dir.is_none());
        assert!(!cli.is_verbose);
        assert!(cli.host.is_none());
        assert!(cli.port.is_none());
        assert!(cli.env.is_none());
    }

    #[test]
    fn test_env_flag() {
        let cli = Cli::parse_from(vec!["-e", "DEV"]).unwrap();
        assert_eq!(cli.env.as_deref(), Some("DEV"));

        let cli2 = Cli::parse_from(vec!["--env", "PROD"]).unwrap();
        assert_eq!(cli2.env.as_deref(), Some("PROD"));

        let cli3 = Cli::parse_from(vec!["--env=development"]).unwrap();
        assert_eq!(cli3.env.as_deref(), Some("development"));
    }

    #[test]
    fn test_api_subcommand() {
        let cli = Cli::parse_from(vec!["api"]).unwrap();
        assert!(cli.api_only);
    }

    #[test]
    fn test_static_dir_override() {
        let cli = Cli::parse_from(vec!["--static-dir", "./custom-dist"]).unwrap();
        assert_eq!(cli.static_dir, Some(PathBuf::from("./custom-dist")));
        assert!(!cli.api_only);
    }

    #[test]
    fn test_static_dir_equals_syntax() {
        let cli = Cli::parse_from(vec!["--static-dir=./abc"]).unwrap();
        assert_eq!(cli.static_dir, Some(PathBuf::from("./abc")));
    }

    #[test]
    fn test_verbose_flag() {
        let cli = Cli::parse_from(vec!["-v"]).unwrap();
        assert!(cli.is_verbose);

        let cli2 = Cli::parse_from(vec!["--verbose"]).unwrap();
        assert!(cli2.is_verbose);
    }

    #[test]
    fn test_host_flag() {
        let cli = Cli::parse_from(vec!["-H", "127.0.0.1"]).unwrap();
        assert_eq!(cli.host.as_deref(), Some("127.0.0.1"));

        let cli2 = Cli::parse_from(vec!["--host", "0.0.0.0"]).unwrap();
        assert_eq!(cli2.host.as_deref(), Some("0.0.0.0"));

        let cli3 = Cli::parse_from(vec!["--host=localhost"]).unwrap();
        assert_eq!(cli3.host.as_deref(), Some("localhost"));
    }

    #[test]
    fn test_port_flag() {
        let cli = Cli::parse_from(vec!["--port", "4000"]).unwrap();
        assert_eq!(cli.port, Some(4000));
    }

    #[test]
    fn test_combined_host_port_verbose() {
        let cli = Cli::parse_from(vec!["--host", "127.0.0.1", "-p", "5171", "-v"]).unwrap();
        assert_eq!(cli.host.as_deref(), Some("127.0.0.1"));
        assert_eq!(cli.port, Some(5171));
        assert!(cli.is_verbose);
    }

    #[test]
    fn test_combined_api_and_verbose() {
        let cli = Cli::parse_from(vec!["api", "-v"]).unwrap();
        assert!(cli.api_only);
        assert!(cli.is_verbose);
    }

    #[test]
    fn test_positional_env() {
        let cli = Cli::parse_from(vec!["DEV"]).unwrap();
        assert_eq!(cli.env.as_deref(), Some("DEV"));

        let cli2 = Cli::parse_from(vec!["prod", "api"]).unwrap();
        assert_eq!(cli2.env.as_deref(), Some("prod"));
        assert!(cli2.api_only);
    }

    #[test]
    fn test_api_flag() {
        let cli = Cli::parse_from(vec!["--api"]).unwrap();
        assert!(cli.api_only);
    }

    #[test]
    fn test_empty_argument_ignored() {
        let cli = Cli::parse_from(vec!["", "   ", "-v"]).unwrap();
        assert!(cli.is_verbose);
    }
}
