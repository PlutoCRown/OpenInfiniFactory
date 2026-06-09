//! Cross-platform key-value text storage.
//!
//! Desktop builds map keys to files under the saves directory. Web builds use
//! `localStorage` (see [Bevy discussion on wasm persistence](https://github.com/bevyengine/bevy/discussions/19188)).

const CONFIG_KEY: &str = "config";
pub const SAVE_PREFIX: &str = "save/";

pub fn read(key: &str) -> Option<String> {
    backend::read(key)
}

pub fn write(key: &str, value: &str) -> bool {
    backend::write(key, value)
}

pub fn remove(key: &str) -> bool {
    backend::remove(key)
}

pub fn exists(key: &str) -> bool {
    backend::exists(key)
}

/// Lists logical keys sharing `prefix`, without the prefix itself.
/// Example: prefix `save/` returns `["world_1", "puzzle_a"]`.
pub fn list_under_prefix(prefix: &str) -> Vec<String> {
    backend::list_under_prefix(prefix)
}

pub fn save_storage_key(name: &str) -> String {
    format!("{SAVE_PREFIX}{name}")
}

pub fn config_key() -> &'static str {
    CONFIG_KEY
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use std::fs;
    use std::path::PathBuf;

    const CONFIG_FILE: &str = "config.ron";

    use super::{CONFIG_KEY, SAVE_PREFIX};
    use crate::shared::platform::saves_directory;

    fn file_path(key: &str) -> PathBuf {
        if key == CONFIG_KEY {
            saves_directory().join(CONFIG_FILE)
        } else if let Some(name) = key.strip_prefix(SAVE_PREFIX) {
            saves_directory().join(format!("{name}.ron"))
        } else {
            saves_directory().join(format!("{key}.ron"))
        }
    }

    pub fn read(key: &str) -> Option<String> {
        fs::read_to_string(file_path(key)).ok()
    }

    pub fn write(key: &str, value: &str) -> bool {
        let path = file_path(key);
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                bevy::log::warn!("Failed to create storage directory: {error}");
                return false;
            }
        }
        match fs::write(path, value) {
            Ok(()) => true,
            Err(error) => {
                bevy::log::warn!("Failed to write storage key {key}: {error}");
                false
            }
        }
    }

    pub fn remove(key: &str) -> bool {
        match fs::remove_file(file_path(key)) {
            Ok(()) => true,
            Err(error) => {
                bevy::log::warn!("Failed to remove storage key {key}: {error}");
                false
            }
        }
    }

    pub fn exists(key: &str) -> bool {
        file_path(key).is_file()
    }

    pub fn list_under_prefix(prefix: &str) -> Vec<String> {
        if prefix != SAVE_PREFIX {
            return Vec::new();
        }

        let dir = saves_directory();
        let Ok(entries) = fs::read_dir(dir) else {
            return Vec::new();
        };

        let mut keys = Vec::new();
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("ron") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if stem == CONFIG_KEY {
                continue;
            }
            keys.push(stem.to_string());
        }
        keys.sort();
        keys
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    const STORAGE_PREFIX: &str = "open_infinifactory:";

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    fn full_key(key: &str) -> String {
        format!("{STORAGE_PREFIX}{key}")
    }

    pub fn read(key: &str) -> Option<String> {
        storage()?.get_item(&full_key(key)).ok()?
    }

    pub fn write(key: &str, value: &str) -> bool {
        match storage() {
            Some(storage) => storage.set_item(&full_key(key), value).is_ok(),
            None => false,
        }
    }

    pub fn remove(key: &str) -> bool {
        match storage() {
            Some(storage) => storage.remove_item(&full_key(key)).is_ok(),
            None => false,
        }
    }

    pub fn exists(key: &str) -> bool {
        read(key).is_some()
    }

    pub fn list_under_prefix(prefix: &str) -> Vec<String> {
        let Some(storage) = storage() else {
            return Vec::new();
        };
        let Ok(length) = storage.length() else {
            return Vec::new();
        };

        let mut keys = Vec::new();
        for index in 0..length {
            let Ok(Some(full_key)) = storage.key(index) else {
                continue;
            };
            let Some(relative) = full_key.strip_prefix(STORAGE_PREFIX) else {
                continue;
            };
            if let Some(name) = relative.strip_prefix(prefix) {
                keys.push(name.to_string());
            }
        }
        keys.sort();
        keys
    }
}
