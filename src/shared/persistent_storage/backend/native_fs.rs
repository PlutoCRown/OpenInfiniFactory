//! 原生文件系统后端（桌面 / iOS / Android 共用；沙盒根目录由 platform 解析）。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::shared::persistent_storage::keys::{CONFIG_KEY, SAVE_PREFIX};
use crate::shared::persistent_storage::vault::PersistOp;
use crate::shared::platform::saves_directory;

/// 把 saves 目录与 config 灌进内存镜像
pub fn load_all() -> HashMap<String, Vec<u8>> {
    let mut entries = HashMap::new();
    let root = saves_directory();

    let config_path = config_path();
    if let Ok(bytes) = fs::read(&config_path) {
        entries.insert(CONFIG_KEY.to_string(), bytes);
    }

    if let Ok(walker) = fs::read_dir(root) {
        for entry in walker.filter_map(Result::ok) {
            collect_tree(&entry.path(), root, &mut entries);
        }
    }
    entries
}

/// 异步落盘一批变更
pub fn apply_ops(ops: &[PersistOp]) {
    for op in ops {
        match op {
            PersistOp::Put { key, value } => {
                let path = key_to_path(key);
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Err(error) = fs::write(&path, value) {
                    bevy::log::warn!("storage put failed `{key}`: {error}");
                }
            }
            PersistOp::RemovePrefix { prefix } => {
                if let Some(relative) = prefix.strip_prefix(SAVE_PREFIX) {
                    let relative = relative.trim_end_matches('/');
                    let path = saves_directory().join(relative);
                    if path.is_dir() {
                        if let Err(error) = fs::remove_dir_all(&path) {
                            bevy::log::warn!("storage remove_prefix `{prefix}`: {error}");
                        }
                    }
                } else {
                    bevy::log::warn!("storage remove_prefix unsupported shape `{prefix}`");
                }
            }
        }
    }
}

fn config_path() -> PathBuf {
    crate::shared::persistent_storage::config_path_resolved()
}

fn key_to_path(key: &str) -> PathBuf {
    if key == CONFIG_KEY {
        return config_path();
    }
    if let Some(rest) = key.strip_prefix(SAVE_PREFIX) {
        return saves_directory().join(rest);
    }
    saves_directory().join(key)
}

fn collect_tree(path: &Path, root: &Path, out: &mut HashMap<String, Vec<u8>>) {
    if path.is_dir() {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.filter_map(Result::ok) {
            collect_tree(&entry.path(), root, out);
        }
        return;
    }
    if !path.is_file() {
        return;
    }
    let Ok(relative) = path.strip_prefix(root) else {
        return;
    };
    let relative = relative.to_string_lossy().replace('\\', "/");
    // 跳过顶层非存档文件里除 config 外的杂项；config 不在 save 树里
    if relative == "config.ron" {
        return;
    }
    let Ok(bytes) = fs::read(path) else {
        return;
    };
    out.insert(format!("{SAVE_PREFIX}{relative}"), bytes);
}
