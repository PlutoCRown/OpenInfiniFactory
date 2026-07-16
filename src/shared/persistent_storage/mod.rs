//! 跨平台持久化：内存镜像 + 异步落盘（桌面/移动 FS，Web IndexedDB）。

mod backend;
mod keys;
mod runtime;
mod vault;

pub use keys::CONFIG_KEY;
pub use runtime::{StoragePlugin, StorageReady};
pub use vault::is_ready;

use std::path::PathBuf;
use std::sync::OnceLock;

use keys::{save_file_key, save_folder_prefix, META_FILE, SAVE_PREFIX};
use vault::lock_vault;

#[cfg(not(target_arch = "wasm32"))]
static CONFIG_PATH_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// 启动参数 `--config` 覆盖默认配置路径（须在首次读写前调用）
pub fn set_config_path_override(path: impl Into<PathBuf>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = CONFIG_PATH_OVERRIDE.set(path.into());
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = path.into();
    }
}

/// 解析后的 config.ron 路径（仅原生）
#[cfg(not(target_arch = "wasm32"))]
pub fn config_path_resolved() -> PathBuf {
    CONFIG_PATH_OVERRIDE
        .get()
        .cloned()
        .unwrap_or_else(|| crate::shared::platform::saves_directory().join("config.ron"))
}

pub fn config_key() -> &'static str {
    CONFIG_KEY
}

pub fn read(key: &str) -> Option<String> {
    lock_vault().get_text(key)
}

pub fn write(key: &str, value: &str) -> bool {
    lock_vault().put_text(key.to_string(), value);
    true
}

pub fn read_save_bytes(save_name: &str, file: &str) -> Option<Vec<u8>> {
    lock_vault()
        .get(&save_file_key(save_name, file))
        .map(|bytes| bytes.to_vec())
}

pub fn write_save_bytes(save_name: &str, file: &str, value: &[u8]) -> bool {
    lock_vault().put(save_file_key(save_name, file), value.to_vec());
    true
}

pub fn read_save_text(save_name: &str, file: &str) -> Option<String> {
    lock_vault().get_text(&save_file_key(save_name, file))
}

pub fn write_save_text(save_name: &str, file: &str, value: &str) -> bool {
    lock_vault().put_text(save_file_key(save_name, file), value);
    true
}

pub fn save_exists(save_name: &str) -> bool {
    lock_vault()
        .get(&save_file_key(save_name, META_FILE))
        .is_some()
}

pub fn remove_save_folder(save_name: &str) -> bool {
    lock_vault().remove_prefix(&save_folder_prefix(save_name))
}

pub fn rename_save_folder(old_name: &str, new_name: &str) -> bool {
    if save_exists(new_name) {
        return false;
    }
    lock_vault().rename_prefix(&save_folder_prefix(old_name), &save_folder_prefix(new_name))
}

pub fn list_saves() -> Vec<String> {
    list_puzzles()
}

/// 顶层 Puzzle 存档名
pub fn list_puzzles() -> Vec<String> {
    let vault = lock_vault();
    let prefix = SAVE_PREFIX;
    let mut names = Vec::new();
    for key in vault.keys_with_prefix(prefix) {
        let Some(relative) = key.strip_prefix(prefix) else {
            continue;
        };
        let Some((save_name, file)) = relative.split_once('/') else {
            continue;
        };
        if save_name.contains('/') || file != META_FILE {
            continue;
        }
        if save_meta_kind_in_vault(&vault, save_name) == Some("puzzle")
            && !names.iter().any(|name| name == save_name)
        {
            names.push(save_name.to_string());
        }
    }
    names.sort();
    names
}

/// 某 Puzzle 下 Solution 名
pub fn list_solution_names(puzzle: &str) -> Vec<String> {
    let vault = lock_vault();
    let prefix = format!("{SAVE_PREFIX}{puzzle}/solutions/");
    let mut names = Vec::new();
    for key in vault.keys_with_prefix(&prefix) {
        let Some(relative) = key.strip_prefix(&prefix) else {
            continue;
        };
        let Some((solution_name, file)) = relative.split_once('/') else {
            continue;
        };
        if solution_name.contains('/') || file != META_FILE {
            continue;
        }
        let storage_path = format!("{puzzle}/solutions/{solution_name}");
        if save_meta_kind_in_vault(&vault, &storage_path) == Some("solution")
            && !names.iter().any(|name| name == solution_name)
        {
            names.push(solution_name.to_string());
        }
    }
    names.sort();
    names
}

fn save_meta_kind_in_vault(
    vault: &vault::MemoryVault,
    storage_path: &str,
) -> Option<&'static str> {
    let text = vault.get_text(&save_file_key(storage_path, META_FILE))?;
    let meta: serde_json::Value = serde_json::from_str(&text).ok()?;
    match meta.get("kind")?.as_str()? {
        "puzzle" => Some("puzzle"),
        "solution" => Some("solution"),
        _ => None,
    }
}
