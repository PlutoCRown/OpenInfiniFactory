#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

const ASSET_DIR_NAME: &str = "assets";
pub const SAVE_DIR: &str = "saves";
#[cfg(not(target_arch = "wasm32"))]
const ASSET_DIR_ENV: &str = "OPEN_INFINIFACTORY_ASSET_DIR";

/// 运行时平台类别（存储 / 沙盒路径分流）
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoragePlatform {
    Desktop,
    Android,
    Ios,
    Web,
}

impl StoragePlatform {
    pub fn current() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::Web
        }
        #[cfg(all(not(target_arch = "wasm32"), target_os = "android"))]
        {
            Self::Android
        }
        #[cfg(all(not(target_arch = "wasm32"), target_os = "ios"))]
        {
            Self::Ios
        }
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        {
            Self::Desktop
        }
    }
}

pub fn asset_path() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        ASSET_DIR_NAME.to_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    resolve_asset_path().to_string_lossy().into_owned()
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_asset_path() -> PathBuf {
    if let Some(path) = asset_path_from_env() {
        return path;
    }

    for candidate in asset_path_candidates() {
        if candidate.is_dir() {
            return candidate;
        }
    }

    PathBuf::from(ASSET_DIR_NAME)
}

#[cfg(not(target_arch = "wasm32"))]
fn asset_path_from_env() -> Option<PathBuf> {
    let path = std::env::var_os(ASSET_DIR_ENV)?;
    let path = PathBuf::from(path);
    path.is_dir().then_some(path)
}

#[cfg(not(target_arch = "wasm32"))]
fn asset_path_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(app_bundle_resource_path(exe_dir));
            candidates.push(exe_dir.join(ASSET_DIR_NAME));
            candidates.push(exe_dir.join("..").join(ASSET_DIR_NAME));
            candidates.push(exe_dir.join("..").join("..").join(ASSET_DIR_NAME));
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join(ASSET_DIR_NAME));
    }

    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ASSET_DIR_NAME));
    candidates
}

#[cfg(all(target_os = "macos", not(target_arch = "wasm32")))]
fn app_bundle_resource_path(exe_dir: &std::path::Path) -> PathBuf {
    exe_dir.join("..").join("Resources").join(ASSET_DIR_NAME)
}

#[cfg(all(not(target_os = "macos"), not(target_arch = "wasm32")))]
fn app_bundle_resource_path(exe_dir: &std::path::Path) -> PathBuf {
    exe_dir.join(ASSET_DIR_NAME)
}

/// 存档根目录。桌面优先 `./saves`；Android/iOS 预留应用沙盒路径（打包时再接到系统 API）。
#[cfg(not(target_arch = "wasm32"))]
pub fn saves_directory() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| match StoragePlatform::current() {
        StoragePlatform::Android | StoragePlatform::Ios => mobile_saves_directory(),
        StoragePlatform::Desktop => desktop_saves_directory(),
        StoragePlatform::Web => PathBuf::from(SAVE_DIR),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn desktop_saves_directory() -> PathBuf {
    let cwd_dir = PathBuf::from(SAVE_DIR);
    if cwd_dir.is_dir() {
        return cwd_dir.canonicalize().unwrap_or(cwd_dir);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            for ancestor in exe_dir.ancestors().take(6) {
                let candidate = ancestor.join(SAVE_DIR);
                if candidate.is_dir() {
                    return candidate.canonicalize().unwrap_or(candidate);
                }
            }
        }
    }

    cwd_dir
}

/// 移动端沙盒存档根：当前回退到 `saves/`，接入 Bevy 移动打包后再换成系统 Documents / files 目录
#[cfg(not(target_arch = "wasm32"))]
fn mobile_saves_directory() -> PathBuf {
    if let Ok(override_dir) = std::env::var("OPEN_INFINIFACTORY_SAVES_DIR") {
        let path = PathBuf::from(override_dir);
        if !path.as_os_str().is_empty() {
            return path;
        }
    }
    desktop_saves_directory()
}
