use bevy::prelude::*;

pub const DEFAULT_DEBUG_HTTP_PORT: u16 = 8765;

#[derive(Resource, Clone, Debug, Default)]
pub struct LaunchOptions {
    pub debug_http_port: Option<u16>,
    pub load_save: Option<String>,
    pub load_fixture: Option<String>,
}

impl LaunchOptions {
    pub fn from_args() -> Self {
        let mut options = Self::default();
        let mut args = std::env::args().skip(1).peekable();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => {
                    print_launch_help();
                    std::process::exit(0);
                }
                "--debug-http" => {
                    options.debug_http_port = Some(DEFAULT_DEBUG_HTTP_PORT);
                }
                "--debug-http-port" => {
                    let Some(value) = args.next() else {
                        eprintln!("error: --debug-http-port requires a port number");
                        print_launch_help();
                        std::process::exit(2);
                    };
                    options.debug_http_port = Some(parse_port(&value));
                }
                value if value.starts_with("--debug-http=") => {
                    let port = value.trim_start_matches("--debug-http=");
                    options.debug_http_port = Some(parse_port(port));
                }
                "--load-save" => {
                    let Some(value) = args.next() else {
                        eprintln!("error: --load-save requires a save name");
                        print_launch_help();
                        std::process::exit(2);
                    };
                    options.load_save = Some(value);
                }
                value if value.starts_with("--load-save=") => {
                    options.load_save = Some(value.trim_start_matches("--load-save=").into());
                }
                "--load-fixture" => {
                    let Some(value) = args.next() else {
                        eprintln!("error: --load-fixture requires a fixture path");
                        print_launch_help();
                        std::process::exit(2);
                    };
                    options.load_fixture = Some(value);
                }
                value if value.starts_with("--load-fixture=") => {
                    options.load_fixture = Some(value.trim_start_matches("--load-fixture=").into());
                }
                _ => {}
            }
        }

        options
    }

    pub fn debug_http_enabled(&self) -> bool {
        self.debug_http_port.is_some()
    }
}

fn parse_port(value: &str) -> u16 {
    match value.parse::<u16>() {
        Ok(port) if port > 0 => port,
        _ => {
            eprintln!("error: invalid debug HTTP port `{value}`");
            print_launch_help();
            std::process::exit(2);
        }
    }
}

fn print_launch_help() {
    eprintln!(
        "\
OpenInfiniFactory

Usage:
  open_infinifactory [OPTIONS]

Options:
  --debug-http              Start local debug HTTP on 127.0.0.1:{DEFAULT_DEBUG_HTTP_PORT} (in-game)
  --debug-http=PORT         Start local debug HTTP on 127.0.0.1:PORT
  --debug-http-port PORT    Same as --debug-http=PORT
  --load-save=NAME          Load save on headless startup (oif-debug-http)
  --load-fixture=PATH       Apply fixture on headless startup (oif-debug-http)
  -h, --help                Show this help

Headless debug server (no window):
  cargo run --bin oif-debug-http [-- --debug-http=PORT] [--load-fixture=blocks/platform.json]

E2E block tests:
  cd e2e && bun test
"
    );
}
