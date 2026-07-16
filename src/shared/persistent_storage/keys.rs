//! 存储键名约定：与平台无关的逻辑路径。

pub const CONFIG_KEY: &str = "config";
pub const SAVE_PREFIX: &str = "save/";

pub const META_FILE: &str = "meta.json";

/// 拼出存档内文件的逻辑键
pub fn save_file_key(save_name: &str, file: &str) -> String {
    format!("{SAVE_PREFIX}{save_name}/{file}")
}

/// 某存档目录下所有文件的前缀
pub fn save_folder_prefix(save_name: &str) -> String {
    format!("{SAVE_PREFIX}{save_name}/")
}
