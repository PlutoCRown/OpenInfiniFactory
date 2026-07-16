use bevy::prelude::*;
use std::path::{Path, PathBuf};

use crate::shared::i18n::Language;
use crate::shared::persistent_storage;
#[cfg(not(target_arch = "wasm32"))]
use crate::shared::platform::saves_directory;
use crate::shared::save::SaveSlot;

pub const DEFAULT_DEBUG_HTTP_PORT: u16 = 8765;

/// 命令行启动选项（游戏客户端与无头 debug 共用）
#[derive(Resource, Clone, Debug, Default)]
pub struct LaunchOptions {
    pub debug_http_port: Option<u16>,
    pub load_save: Option<String>,
    pub load_fixture: Option<String>,
    pub config_path: Option<PathBuf>,
    pub language: Option<Language>,
    /// 强制启用虚拟遥感（桌面调试用）
    pub force_touch: bool,
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
                        eprintln!("error: --load-save requires a save path");
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
                "--config" => {
                    let Some(value) = args.next() else {
                        eprintln!("error: --config requires a file path");
                        print_launch_help();
                        std::process::exit(2);
                    };
                    options.config_path = Some(PathBuf::from(value));
                }
                value if value.starts_with("--config=") => {
                    options.config_path =
                        Some(PathBuf::from(value.trim_start_matches("--config=")));
                }
                "--language" => {
                    let Some(value) = args.next() else {
                        eprintln!("error: --language requires en or zh-CN");
                        print_launch_help();
                        std::process::exit(2);
                    };
                    options.language = Some(parse_language(&value));
                }
                value if value.starts_with("--language=") => {
                    options.language =
                        Some(parse_language(value.trim_start_matches("--language=")));
                }
                "--touch" | "--virtual-remote" => {
                    options.force_touch = true;
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

/// 把 `--load-save` 参数解析成存档槽位（相对路径或绝对路径均可）
pub fn resolve_launch_save_slot(raw: &str) -> Option<SaveSlot> {
    let normalized = normalize_save_arg(raw)?;
    let slot = SaveSlot::from_storage_path(&normalized)?;
    persistent_storage::save_exists(&slot.storage_path()).then_some(slot)
}

fn normalize_save_arg(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches('"');
    if trimmed.is_empty() {
        return None;
    }
    let path = Path::new(trimmed);
    let relative = if path.is_absolute() {
        #[cfg(target_arch = "wasm32")]
        {
            // Web 无本地 saves 目录，不接受绝对路径
            return None;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let saves = saves_directory();
            path.canonicalize()
                .ok()
                .and_then(|canon| {
                    canon
                        .strip_prefix(saves)
                        .ok()
                        .map(|rest| rest.to_path_buf())
                })
                .or_else(|| path.strip_prefix(saves).ok().map(|rest| rest.to_path_buf()))?
                .to_string_lossy()
                .into_owned()
        }
    } else {
        trimmed
            .trim_start_matches("./")
            .strip_prefix("saves/")
            .unwrap_or(trimmed.trim_start_matches("./"))
            .to_string()
    };
    Some(relative.replace('\\', "/"))
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

fn parse_language(value: &str) -> Language {
    match value.trim().to_ascii_lowercase().as_str() {
        "en" | "english" => Language::English,
        "zh" | "zh-cn" | "zh_cn" | "chinese" | "cn" => Language::ChineseSimplified,
        _ => {
            eprintln!("error: invalid language `{value}` (use en or zh-CN)");
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
  --config=PATH             Use this config.ron instead of saves/config.ron
  --language=en|zh-CN       Override UI language for this launch
  --touch, --virtual-remote Force virtual on-screen controls (desktop debug)
  --load-save=PATH          Load save and enter world (game client + headless)
                            Examples: Important_Test
                                      Important_Test/solutions/Solution1
                                      saves/Important_Test/solutions/Solution1
  --load-fixture=PATH       Apply fixture on headless startup (oif-debug-http)
  -h, --help                Show this help

Headless debug server (no window):
  cargo run --bin oif-debug-http [-- --debug-http=PORT] [--load-fixture=blocks/platform.json]

E2E block tests:
  cd e2e && bun test
"
    );
}
