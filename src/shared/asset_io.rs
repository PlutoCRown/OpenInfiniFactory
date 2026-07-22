//! 游戏资源读写：桌面/存档用文件系统；Android 打包资源走 AssetManager

use std::path::{Path, PathBuf};

/// 读资源文件全部字节
pub fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    #[cfg(target_os = "android")]
    if let Some(key) = android_asset_key(path) {
        return android_read_bytes(&key)
            .map_err(|e| format!("read asset `{}` ({}): {e}", path.display(), key));
    }
    std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))
}

/// 读资源文件为 UTF-8 文本
pub fn read_to_string(path: &Path) -> Result<String, String> {
    let bytes = read_bytes(path)?;
    String::from_utf8(bytes).map_err(|e| format!("utf8 {}: {e}", path.display()))
}

/// 是否为可读文件
pub fn is_file(path: &Path) -> bool {
    #[cfg(target_os = "android")]
    if let Some(key) = android_asset_key(path) {
        return android_is_file(&key);
    }
    path.is_file()
}

/// 是否为可列目录
pub fn is_dir(path: &Path) -> bool {
    #[cfg(target_os = "android")]
    if let Some(key) = android_asset_key(path) {
        return android_is_dir(&key);
    }
    path.is_dir()
}

/// 列出子目录（按目录名排序）；根不存在则空列表
pub fn list_subdirs(path: &Path) -> Result<Vec<PathBuf>, String> {
    if !is_dir(path) {
        return Ok(Vec::new());
    }

    #[cfg(target_os = "android")]
    if let Some(key) = android_asset_key(path) {
        let names = android_list_names(&key)
            .map_err(|e| format!("list asset `{}` ({}): {e}", path.display(), key))?;
        let mut dirs: Vec<PathBuf> = names
            .into_iter()
            .map(|name| path.join(name))
            .filter(|child| is_dir(child))
            .collect();
        dirs.sort_by(|a, b| {
            let a_name = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let b_name = b.file_name().and_then(|s| s.to_str()).unwrap_or("");
            a_name.cmp(b_name)
        });
        return Ok(dirs);
    }

    let mut dirs: Vec<PathBuf> = std::fs::read_dir(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|child| child.is_dir())
        .collect();
    dirs.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|s| s.to_str()).unwrap_or("");
        a_name.cmp(b_name)
    });
    Ok(dirs)
}

/// 若路径指向 APK 内打包资源，返回 AssetManager 相对键；绝对路径（如存档）返回 None
#[cfg(target_os = "android")]
fn android_asset_key(path: &Path) -> Option<String> {
    if path.is_absolute() {
        return None;
    }
    let raw = path.to_string_lossy().replace('\\', "/");
    let trimmed = raw
        .strip_prefix("assets/")
        .or_else(|| raw.strip_prefix("./assets/"))
        .unwrap_or(raw.as_str())
        .trim_start_matches('/');
    // `assets` 自身 → 根目录键为空串
    if trimmed == "assets" || trimmed == "." {
        return Some(String::new());
    }
    Some(trimmed.to_string())
}

#[cfg(target_os = "android")]
fn android_app() -> Result<&'static bevy::android::android_activity::AndroidApp, String> {
    bevy::android::ANDROID_APP
        .get()
        .ok_or_else(|| "ANDROID_APP not set".to_string())
}

#[cfg(target_os = "android")]
fn android_read_bytes(key: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;

    let manager = android_app()?.asset_manager();
    let c_key = std::ffi::CString::new(key).map_err(|e| e.to_string())?;
    let mut asset = manager
        .open(&c_key)
        .ok_or_else(|| "AssetManager.open failed".to_string())?;
    if let Ok(buf) = asset.buffer() {
        return Ok(buf.to_vec());
    }
    let mut bytes = Vec::new();
    asset
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Asset read: {e}"))?;
    Ok(bytes)
}

#[cfg(target_os = "android")]
fn android_is_file(key: &str) -> bool {
    let Ok(app) = android_app() else {
        return false;
    };
    let Ok(c_key) = std::ffi::CString::new(key) else {
        return false;
    };
    app.asset_manager().open(&c_key).is_some()
}

#[cfg(target_os = "android")]
fn android_is_dir(key: &str) -> bool {
    let Ok(app) = android_app() else {
        return false;
    };
    let Ok(c_key) = std::ffi::CString::new(key) else {
        return false;
    };
    let manager = app.asset_manager();
    // 与 Bevy AndroidAssetReader 相同：open_dir 能开、open 失败 → 目录
    manager.open_dir(&c_key).is_some() && manager.open(&c_key).is_none()
}

#[cfg(target_os = "android")]
fn android_list_names(key: &str) -> Result<Vec<String>, String> {
    let manager = android_app()?.asset_manager();
    let c_key = std::ffi::CString::new(key).map_err(|e| e.to_string())?;
    let dir = manager
        .open_dir(&c_key)
        .ok_or_else(|| "AssetManager.open_dir failed".to_string())?;
    let mut names = Vec::new();
    for name in dir {
        let Some(s) = name.to_str().ok().map(str::to_owned) else {
            continue;
        };
        if s.is_empty() {
            continue;
        }
        names.push(s);
    }
    Ok(names)
}
