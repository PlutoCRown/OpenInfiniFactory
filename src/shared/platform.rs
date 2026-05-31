use std::path::PathBuf;

const ASSET_DIR_NAME: &str = "assets";
const ASSET_DIR_ENV: &str = "OPEN_INFINIFACTORY_ASSET_DIR";

pub fn asset_path() -> String {
    resolve_asset_path()
        .to_string_lossy()
        .into_owned()
}

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

fn asset_path_from_env() -> Option<PathBuf> {
    let path = std::env::var_os(ASSET_DIR_ENV)?;
    let path = PathBuf::from(path);
    path.is_dir().then_some(path)
}

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

#[cfg(target_os = "macos")]
fn app_bundle_resource_path(exe_dir: &std::path::Path) -> PathBuf {
    exe_dir.join("..").join("Resources").join(ASSET_DIR_NAME)
}

#[cfg(not(target_os = "macos"))]
fn app_bundle_resource_path(exe_dir: &std::path::Path) -> PathBuf {
    exe_dir.join(ASSET_DIR_NAME)
}
