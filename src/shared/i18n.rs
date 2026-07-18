use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_LANGUAGE: Language = Language::English;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub enum Language {
    English,
    ChineseSimplified,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::English, Language::ChineseSimplified];

    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::ChineseSimplified => "zh-CN",
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::ChineseSimplified => "简体中文",
        }
    }
}

/// 启动时按配置加载的文案表；会话内语言不变，热路径用 `t`/`fmt_into` 避免多余分配
#[derive(Resource)]
pub struct I18n {
    language: Language,
    messages: HashMap<String, String>,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            messages: load_messages(language),
        }
    }

    pub fn language(&self) -> Language {
        self.language
    }

    /// 查表返回借用，不 clone；缺失时回退为 key 本身
    pub fn t(&self, key: &'static str) -> &str {
        self.messages
            .get(key)
            .map(String::as_str)
            .unwrap_or(key)
    }

    /// 低频：拼出新 String；热路径请用 `fmt_into`
    pub fn fmt(&self, key: &'static str, values: &[(&str, &str)]) -> String {
        let mut out = String::new();
        self.fmt_into(&mut out, key, values);
        out
    }

    /// 把模板填进 `out`（会先 clear），占位符为 `{name}`，不做中间 String::replace
    pub fn fmt_into(&self, out: &mut String, key: &'static str, values: &[(&str, &str)]) {
        subst_template(self.t(key), values, out);
    }
}

/// 扫描 `{name}` 占位符写入 out
pub fn subst_template(template: &str, values: &[(&str, &str)], out: &mut String) {
    out.clear();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[start..]);
            return;
        };
        let name = &after[..end];
        if let Some((_, value)) = values.iter().find(|(n, _)| *n == name) {
            out.push_str(value);
        } else {
            out.push('{');
            out.push_str(name);
            out.push('}');
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
}

fn load_messages(language: Language) -> HashMap<String, String> {
    serde_json::from_str(language_json(language)).unwrap_or_else(|error| {
        warn!("Failed to load {} locale: {error}", language.code());
        serde_json::from_str(language_json(DEFAULT_LANGUAGE)).unwrap_or_default()
    })
}

fn language_json(language: Language) -> &'static str {
    match language {
        Language::English => include_str!("../../assets/i18n/en.json"),
        Language::ChineseSimplified => include_str!("../../assets/i18n/zh-CN.json"),
    }
}

pub fn resolve_language(user_language: Option<Language>) -> Language {
    user_language
        .or_else(detect_system_language)
        .unwrap_or(DEFAULT_LANGUAGE)
}

pub fn detect_system_language() -> Option<Language> {
    platform_language_tag().and_then(language_from_tag)
}

fn language_from_tag(tag: String) -> Option<Language> {
    let normalized = tag.replace('_', "-").to_ascii_lowercase();
    if normalized.starts_with("zh") {
        Some(Language::ChineseSimplified)
    } else if normalized.starts_with("en") {
        Some(Language::English)
    } else {
        None
    }
}

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
fn platform_language_tag() -> Option<String> {
    env_language_tag().or_else(system_ui_language_tag)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_language_tag() -> Option<String> {
    None
}

fn env_language_tag() -> Option<String> {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key) {
            if is_meaningful_locale(&value) {
                return Some(value);
            }
        }
    }
    None
}

fn is_meaningful_locale(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value.eq_ignore_ascii_case("C")
        && !value.eq_ignore_ascii_case("C.UTF-8")
        && !value.to_ascii_lowercase().starts_with("c.")
}

#[cfg(target_os = "macos")]
fn system_ui_language_tag() -> Option<String> {
    // Agent/Cursor shell 常是 LANG=C.UTF-8，回退读系统界面语言
    std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLocale"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(not(target_os = "macos"))]
fn system_ui_language_tag() -> Option<String> {
    None
}
