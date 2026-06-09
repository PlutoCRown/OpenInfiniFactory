#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

const ASSET_DIR_NAME: &str = "assets";
pub const SAVE_DIR: &str = "saves";
#[cfg(not(target_arch = "wasm32"))]
const ASSET_DIR_ENV: &str = "OPEN_INFINIFACTORY_ASSET_DIR";

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

/// Resolved save directory. Prefers `./saves` under cwd, then walks up from the
/// executable (covers `cargo run` launching from `target/debug/`).
#[cfg(not(target_arch = "wasm32"))]
pub fn saves_directory() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
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
    })
}
