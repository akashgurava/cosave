use std::{env, path::PathBuf};

/// Command-line configuration for the server process.
pub(crate) struct Cli {
    /// Disable static asset hosting and only serve `/api/v1` routes.
    pub(crate) api_only: bool,
    /// Custom directory containing static SPA assets.
    pub(crate) static_dir: Option<PathBuf>,
    /// Turn on verbose/debug logging output.
    pub(crate) is_verbose: bool,
    /// Port to listen on (overrides PORT env, defaults to 3000).
    pub(crate) port: Option<u16>,
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
        let mut api_only = false;
        let mut static_dir = None;
        let mut is_verbose = false;
        let mut port = None;

        let args: Vec<String> = args.into_iter().map(Into::into).collect();
        let mut iter = args.into_iter().peekable();

        while let Some(arg) = iter.next() {
            if arg.trim().is_empty() {
                continue;
            }
            match arg.as_str() {
                "api" => {
                    api_only = true;
                }
                "-v" | "--verbose" | "--debug" => {
                    is_verbose = true;
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
            api_only,
            static_dir,
            is_verbose,
            port,
        })
    }

    fn help_text() -> String {
        r#"CoSave — High-Performance Family Finance Server

Usage:
  cosave [COMMAND] [OPTIONS]

Commands:
  api                    Run in API-only mode (disables static file serving)

Options:
  --static-dir <PATH>    Override the directory for serving frontend static files
  -p, --port <PORT>      Specify the port to listen on (overrides PORT env var, default: 3000)
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
        assert!(cli.port.is_none());
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
    fn test_port_flag() {
        let cli = Cli::parse_from(vec!["--port", "4000"]).unwrap();
        assert_eq!(cli.port, Some(4000));
    }

    #[test]
    fn test_combined_api_and_verbose() {
        let cli = Cli::parse_from(vec!["api", "-v"]).unwrap();
        assert!(cli.api_only);
        assert!(cli.is_verbose);
    }

    #[test]
    fn test_empty_argument_ignored() {
        let cli = Cli::parse_from(vec!["", "   ", "-v"]).unwrap();
        assert!(cli.is_verbose);
    }
}
