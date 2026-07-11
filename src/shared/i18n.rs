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

    pub fn set_language(&mut self, language: Language) {
        if self.language == language {
            return;
        }
        self.language = language;
        self.messages = load_messages(language);
    }

    pub fn text(&self, key: &'static str) -> String {
        self.messages
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback_text(key).to_string())
    }

    pub fn fmt(&self, key: &'static str, values: &[(&str, String)]) -> String {
        let mut text = self.text(key);
        for (name, value) in values {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }
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

fn fallback_text(key: &str) -> &str {
    key
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
