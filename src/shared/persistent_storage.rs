//! 跨平台持久化：config 单文件 + 存档文件夹。

const CONFIG_KEY: &str = "config";
pub const SAVE_PREFIX: &str = "save/";

const META_FILE: &str = "meta.json";

pub fn read(key: &str) -> Option<String> {
    backend::read_config(key)
}

pub fn write(key: &str, value: &str) -> bool {
    backend::write_config(key, value)
}

pub fn config_key() -> &'static str {
    CONFIG_KEY
}

pub fn read_save_bytes(save_name: &str, file: &str) -> Option<Vec<u8>> {
    backend::read_save_bytes(save_name, file)
}

pub fn write_save_bytes(save_name: &str, file: &str, value: &[u8]) -> bool {
    backend::write_save_bytes(save_name, file, value)
}

pub fn read_save_text(save_name: &str, file: &str) -> Option<String> {
    backend::read_save_text(save_name, file)
}

pub fn write_save_text(save_name: &str, file: &str, value: &str) -> bool {
    backend::write_save_text(save_name, file, value)
}

pub fn save_exists(save_name: &str) -> bool {
    backend::save_exists(save_name)
}

pub fn remove_save_folder(save_name: &str) -> bool {
    backend::remove_save_folder(save_name)
}

pub fn rename_save_folder(old_name: &str, new_name: &str) -> bool {
    backend::rename_save_folder(old_name, new_name)
}

pub fn list_saves() -> Vec<String> {
    list_puzzles()
}

/// 顶层 Puzzle 存档名（不含 solutions 子目录）
pub fn list_puzzles() -> Vec<String> {
    backend::list_puzzles()
}

/// 某 Puzzle 下已存的 Solution 名
pub fn list_solution_names(puzzle: &str) -> Vec<String> {
    backend::list_solution_names(puzzle)
}

/// 启动参数 `--config` 覆盖默认 `saves/config.ron`（须在首次读写配置前调用）
pub fn set_config_path_override(path: impl Into<std::path::PathBuf>) {
    #[cfg(not(target_arch = "wasm32"))]
    backend::set_config_path_override(path.into());
    #[cfg(target_arch = "wasm32")]
    let _ = path.into();
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;

    use super::{CONFIG_KEY, META_FILE};
    use crate::shared::platform::saves_directory;

    static CONFIG_PATH_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

    pub fn set_config_path_override(path: PathBuf) {
        let _ = CONFIG_PATH_OVERRIDE.set(path);
    }

    fn config_path() -> PathBuf {
        CONFIG_PATH_OVERRIDE
            .get()
            .cloned()
            .unwrap_or_else(|| saves_directory().join("config.ron"))
    }

    fn save_dir(save_name: &str) -> PathBuf {
        saves_directory().join(save_name)
    }

    pub fn read_config(key: &str) -> Option<String> {
        if key != CONFIG_KEY {
            return None;
        }
        fs::read_to_string(config_path()).ok()
    }

    pub fn write_config(key: &str, value: &str) -> bool {
        if key != CONFIG_KEY {
            return false;
        }
        let path = config_path();
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                bevy::log::warn!("Failed to create storage directory: {error}");
                return false;
            }
        }
        match fs::write(path, value) {
            Ok(()) => true,
            Err(error) => {
                bevy::log::warn!("Failed to write config: {error}");
                false
            }
        }
    }

    pub fn read_save_bytes(save_name: &str, file: &str) -> Option<Vec<u8>> {
        fs::read(save_dir(save_name).join(file)).ok()
    }

    pub fn write_save_bytes(save_name: &str, file: &str, value: &[u8]) -> bool {
        let path = save_dir(save_name).join(file);
        ensure_save_dir(&path) && fs::write(path, value).is_ok()
    }

    pub fn read_save_text(save_name: &str, file: &str) -> Option<String> {
        fs::read_to_string(save_dir(save_name).join(file)).ok()
    }

    pub fn write_save_text(save_name: &str, file: &str, value: &str) -> bool {
        let path = save_dir(save_name).join(file);
        ensure_save_dir(&path) && fs::write(path, value).is_ok()
    }

    pub fn save_exists(save_name: &str) -> bool {
        save_dir(save_name).join(META_FILE).is_file()
    }

    pub fn remove_save_folder(save_name: &str) -> bool {
        let path = save_dir(save_name);
        if !path.is_dir() {
            return false;
        }
        match fs::remove_dir_all(path) {
            Ok(()) => true,
            Err(error) => {
                bevy::log::warn!("Failed to remove save folder {save_name}: {error}");
                false
            }
        }
    }

    pub fn rename_save_folder(old_name: &str, new_name: &str) -> bool {
        let from = save_dir(old_name);
        let to = save_dir(new_name);
        if !from.is_dir() || to.exists() {
            return false;
        }
        match fs::rename(from, to) {
            Ok(()) => true,
            Err(error) => {
                bevy::log::warn!("Failed to rename save {old_name} -> {new_name}: {error}");
                false
            }
        }
    }

    pub fn list_puzzles() -> Vec<String> {
        let dir = saves_directory();
        let Ok(entries) = fs::read_dir(dir) else {
            return Vec::new();
        };

        let mut names = Vec::new();
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if !path.join(META_FILE).is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if save_meta_kind(name) == Some("puzzle") {
                names.push(name.to_string());
            }
        }
        names.sort();
        names
    }

    pub fn list_solution_names(puzzle: &str) -> Vec<String> {
        let solutions_dir = save_dir(puzzle).join("solutions");
        let Ok(entries) = fs::read_dir(solutions_dir) else {
            return Vec::new();
        };

        let mut names = Vec::new();
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() || !path.join(META_FILE).is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if save_meta_kind(&format!("{puzzle}/solutions/{name}")) == Some("solution") {
                names.push(name.to_string());
            }
        }
        names.sort();
        names
    }

    fn save_meta_kind(storage_path: &str) -> Option<&'static str> {
        let text = fs::read_to_string(save_dir(storage_path).join(META_FILE)).ok()?;
        let meta: serde_json::Value = serde_json::from_str(&text).ok()?;
        match meta.get("kind")?.as_str()? {
            "puzzle" => Some("puzzle"),
            "solution" => Some("solution"),
            _ => None,
        }
    }

    fn ensure_save_dir(path: &Path) -> bool {
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                bevy::log::warn!("Failed to create save directory: {error}");
                return false;
            }
        }
        true
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    use super::{META_FILE, SAVE_PREFIX};

    const STORAGE_PREFIX: &str = "open_infinifactory:";

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    fn full_key(relative: &str) -> String {
        format!("{STORAGE_PREFIX}{relative}")
    }

    fn save_key(save_name: &str, file: &str) -> String {
        full_key(&format!("{SAVE_PREFIX}{save_name}/{file}"))
    }

    fn is_binary_file(file: &str) -> bool {
        file.ends_with(".bin") || file.ends_with(".png")
    }

    pub fn read_config(key: &str) -> Option<String> {
        storage()?.get_item(&full_key(key)).ok()?
    }

    pub fn write_config(key: &str, value: &str) -> bool {
        match storage() {
            Some(storage) => storage.set_item(&full_key(key), value).is_ok(),
            None => false,
        }
    }

    pub fn read_save_bytes(save_name: &str, file: &str) -> Option<Vec<u8>> {
        let text = storage()?.get_item(&save_key(save_name, file)).ok()??;
        STANDARD.decode(text).ok()
    }

    pub fn write_save_bytes(save_name: &str, file: &str, value: &[u8]) -> bool {
        let encoded = STANDARD.encode(value);
        storage()
            .and_then(|storage| storage.set_item(&save_key(save_name, file), &encoded).ok())
            .is_some()
    }

    pub fn read_save_text(save_name: &str, file: &str) -> Option<String> {
        storage()?.get_item(&save_key(save_name, file)).ok()?
    }

    pub fn write_save_text(save_name: &str, file: &str, value: &str) -> bool {
        storage()
            .and_then(|storage| storage.set_item(&save_key(save_name, file), value).ok())
            .is_some()
    }

    pub fn save_exists(save_name: &str) -> bool {
        read_save_text(save_name, META_FILE).is_some()
    }

    pub fn remove_save_folder(save_name: &str) -> bool {
        let Some(storage) = storage() else {
            return false;
        };
        let prefix = full_key(&format!("{SAVE_PREFIX}{save_name}/"));
        let Ok(length) = storage.length() else {
            return false;
        };

        let mut keys = Vec::new();
        for index in 0..length {
            let Ok(Some(key)) = storage.key(index) else {
                continue;
            };
            if key.starts_with(&prefix) {
                keys.push(key);
            }
        }
        keys.iter()
            .all(|key| storage.remove_item(key).unwrap_or(false))
    }

    pub fn rename_save_folder(old_name: &str, new_name: &str) -> bool {
        if save_exists(new_name) {
            return false;
        }
        let files = list_save_files(old_name);
        if files.is_empty() {
            return false;
        }
        for file in &files {
            let key = save_key(old_name, file);
            let Some(storage) = storage() else {
                return false;
            };
            let Ok(Some(value)) = storage.get_item(&key) else {
                return false;
            };
            if !write_value(new_name, file, &value) {
                return false;
            }
        }
        remove_save_folder(old_name)
    }

    pub fn list_puzzles() -> Vec<String> {
        let Some(storage) = storage() else {
            return Vec::new();
        };
        let Ok(length) = storage.length() else {
            return Vec::new();
        };

        let prefix = full_key(SAVE_PREFIX);
        let mut names = Vec::new();
        for index in 0..length {
            let Ok(Some(key)) = storage.key(index) else {
                continue;
            };
            let Some(relative) = key.strip_prefix(&prefix) else {
                continue;
            };
            let Some((save_name, file)) = relative.split_once('/') else {
                continue;
            };
            if save_name.contains('/') {
                continue;
            }
            if file == META_FILE && !names.iter().any(|name| name == save_name) {
                names.push(save_name.to_string());
            }
        }
        names.sort();
        names
    }

    pub fn list_solution_names(puzzle: &str) -> Vec<String> {
        let Some(storage) = storage() else {
            return Vec::new();
        };
        let Ok(length) = storage.length() else {
            return Vec::new();
        };

        let prefix = full_key(&format!("{SAVE_PREFIX}{puzzle}/solutions/"));
        let mut names = Vec::new();
        for index in 0..length {
            let Ok(Some(key)) = storage.key(index) else {
                continue;
            };
            let Some(relative) = key.strip_prefix(&prefix) else {
                continue;
            };
            let Some((solution_name, file)) = relative.split_once('/') else {
                continue;
            };
            if solution_name.contains('/') {
                continue;
            }
            if file == META_FILE && !names.iter().any(|name| name == solution_name) {
                names.push(solution_name.to_string());
            }
        }
        names.sort();
        names
    }

    fn list_save_files(save_name: &str) -> Vec<String> {
        let Some(storage) = storage() else {
            return Vec::new();
        };
        let Ok(length) = storage.length() else {
            return Vec::new();
        };
        let prefix = full_key(&format!("{SAVE_PREFIX}{save_name}/"));
        let mut files = Vec::new();
        for index in 0..length {
            let Ok(Some(key)) = storage.key(index) else {
                continue;
            };
            let Some(relative) = key.strip_prefix(&prefix) else {
                continue;
            };
            files.push(relative.to_string());
        }
        files
    }

    fn write_value(save_name: &str, file: &str, value: &str) -> bool {
        if is_binary_file(file) {
            STANDARD
                .decode(value)
                .map(|bytes| write_save_bytes(save_name, file, &bytes))
                .unwrap_or(false)
        } else {
            write_save_text(save_name, file, value)
        }
    }
}
