use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::shared::i18n::Language;
use crate::shared::save::SAVE_DIR;

pub const CONFIG_FILE: &str = "config.ron";

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub fov_degrees: f32,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default)]
    pub language: Option<Language>,
    pub key_bindings: KeyBindings,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            fov_degrees: 70.0,
            ui_scale: default_ui_scale(),
            language: None,
            key_bindings: KeyBindings::default(),
        }
    }
}

fn default_ui_scale() -> f32 {
    1.0
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub pause: ConfigKey,
    pub inventory: ConfigKey,
    pub rotate_or_rollback: ConfigKey,
    pub simulate: ConfigKey,
    pub debug: ConfigKey,
    pub forward: ConfigKey,
    pub backward: ConfigKey,
    pub left: ConfigKey,
    pub right: ConfigKey,
    pub jump_or_fly_up: ConfigKey,
    pub fly_down: ConfigKey,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            pause: ConfigKey::Escape,
            inventory: ConfigKey::KeyE,
            rotate_or_rollback: ConfigKey::KeyR,
            simulate: ConfigKey::KeyF,
            debug: ConfigKey::Slash,
            forward: ConfigKey::KeyW,
            backward: ConfigKey::KeyS,
            left: ConfigKey::KeyA,
            right: ConfigKey::KeyD,
            jump_or_fly_up: ConfigKey::Space,
            fly_down: ConfigKey::ShiftLeft,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConfigAction {
    Pause,
    Inventory,
    RotateOrRollback,
    Simulate,
    Debug,
    Forward,
    Backward,
    Left,
    Right,
    JumpOrFlyUp,
    FlyDown,
}

impl ConfigAction {
    pub const ALL: [ConfigAction; 11] = [
        ConfigAction::Pause,
        ConfigAction::Inventory,
        ConfigAction::RotateOrRollback,
        ConfigAction::Simulate,
        ConfigAction::Debug,
        ConfigAction::Forward,
        ConfigAction::Backward,
        ConfigAction::Left,
        ConfigAction::Right,
        ConfigAction::JumpOrFlyUp,
        ConfigAction::FlyDown,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            ConfigAction::Pause => "action.pause",
            ConfigAction::Inventory => "action.inventory",
            ConfigAction::RotateOrRollback => "action.rotate_or_rollback",
            ConfigAction::Simulate => "action.simulate",
            ConfigAction::Debug => "action.debug",
            ConfigAction::Forward => "action.forward",
            ConfigAction::Backward => "action.backward",
            ConfigAction::Left => "action.left",
            ConfigAction::Right => "action.right",
            ConfigAction::JumpOrFlyUp => "action.jump_or_fly_up",
            ConfigAction::FlyDown => "action.fly_down",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConfigKey {
    Escape,
    Space,
    ShiftLeft,
    ShiftRight,
    Slash,
    KeyA,
    KeyD,
    KeyE,
    KeyF,
    KeyI,
    KeyR,
    KeyS,
    KeyW,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
}

impl ConfigKey {
    pub fn key_code(self) -> KeyCode {
        match self {
            ConfigKey::Escape => KeyCode::Escape,
            ConfigKey::Space => KeyCode::Space,
            ConfigKey::ShiftLeft => KeyCode::ShiftLeft,
            ConfigKey::ShiftRight => KeyCode::ShiftRight,
            ConfigKey::Slash => KeyCode::Slash,
            ConfigKey::KeyA => KeyCode::KeyA,
            ConfigKey::KeyD => KeyCode::KeyD,
            ConfigKey::KeyE => KeyCode::KeyE,
            ConfigKey::KeyF => KeyCode::KeyF,
            ConfigKey::KeyI => KeyCode::KeyI,
            ConfigKey::KeyR => KeyCode::KeyR,
            ConfigKey::KeyS => KeyCode::KeyS,
            ConfigKey::KeyW => KeyCode::KeyW,
            ConfigKey::Digit1 => KeyCode::Digit1,
            ConfigKey::Digit2 => KeyCode::Digit2,
            ConfigKey::Digit3 => KeyCode::Digit3,
            ConfigKey::Digit4 => KeyCode::Digit4,
            ConfigKey::Digit5 => KeyCode::Digit5,
            ConfigKey::Digit6 => KeyCode::Digit6,
            ConfigKey::Digit7 => KeyCode::Digit7,
            ConfigKey::Digit8 => KeyCode::Digit8,
            ConfigKey::Digit9 => KeyCode::Digit9,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ConfigKey::Escape => "Esc",
            ConfigKey::Space => "Space",
            ConfigKey::ShiftLeft => "Left Shift",
            ConfigKey::ShiftRight => "Right Shift",
            ConfigKey::Slash => "/",
            ConfigKey::KeyA => "A",
            ConfigKey::KeyD => "D",
            ConfigKey::KeyE => "E",
            ConfigKey::KeyF => "F",
            ConfigKey::KeyI => "I",
            ConfigKey::KeyR => "R",
            ConfigKey::KeyS => "S",
            ConfigKey::KeyW => "W",
            ConfigKey::Digit1 => "1",
            ConfigKey::Digit2 => "2",
            ConfigKey::Digit3 => "3",
            ConfigKey::Digit4 => "4",
            ConfigKey::Digit5 => "5",
            ConfigKey::Digit6 => "6",
            ConfigKey::Digit7 => "7",
            ConfigKey::Digit8 => "8",
            ConfigKey::Digit9 => "9",
        }
    }
}

impl GameConfig {
    pub fn key(&self, action: ConfigAction) -> ConfigKey {
        match action {
            ConfigAction::Pause => self.key_bindings.pause,
            ConfigAction::Inventory => self.key_bindings.inventory,
            ConfigAction::RotateOrRollback => self.key_bindings.rotate_or_rollback,
            ConfigAction::Simulate => self.key_bindings.simulate,
            ConfigAction::Debug => self.key_bindings.debug,
            ConfigAction::Forward => self.key_bindings.forward,
            ConfigAction::Backward => self.key_bindings.backward,
            ConfigAction::Left => self.key_bindings.left,
            ConfigAction::Right => self.key_bindings.right,
            ConfigAction::JumpOrFlyUp => self.key_bindings.jump_or_fly_up,
            ConfigAction::FlyDown => self.key_bindings.fly_down,
        }
    }

    pub fn set_key(&mut self, action: ConfigAction, key: ConfigKey) {
        match action {
            ConfigAction::Pause => self.key_bindings.pause = key,
            ConfigAction::Inventory => self.key_bindings.inventory = key,
            ConfigAction::RotateOrRollback => self.key_bindings.rotate_or_rollback = key,
            ConfigAction::Simulate => self.key_bindings.simulate = key,
            ConfigAction::Debug => self.key_bindings.debug = key,
            ConfigAction::Forward => self.key_bindings.forward = key,
            ConfigAction::Backward => self.key_bindings.backward = key,
            ConfigAction::Left => self.key_bindings.left = key,
            ConfigAction::Right => self.key_bindings.right = key,
            ConfigAction::JumpOrFlyUp => self.key_bindings.jump_or_fly_up = key,
            ConfigAction::FlyDown => self.key_bindings.fly_down = key,
        }
    }
}

pub fn load_config() -> GameConfig {
    let Ok(contents) = fs::read_to_string(config_path()) else {
        let config = GameConfig::default();
        save_config(&config);
        return config;
    };
    ron::from_str::<GameConfig>(&contents).unwrap_or_else(|error| {
        warn!("Failed to load config: {error}");
        GameConfig::default()
    })
}

pub fn save_config(config: &GameConfig) {
    if let Err(error) = fs::create_dir_all(SAVE_DIR) {
        warn!("Failed to create config directory: {error}");
        return;
    }

    match ron::ser::to_string_pretty(config, PrettyConfig::default()) {
        Ok(serialized) => {
            if let Err(error) = fs::write(config_path(), serialized) {
                warn!("Failed to write config: {error}");
            }
        }
        Err(error) => warn!("Failed to serialize config: {error}"),
    }
}

pub fn open_config_folder() {
    if let Err(error) = fs::create_dir_all(SAVE_DIR) {
        warn!("Failed to create config directory: {error}");
        return;
    }

    if let Err(error) = Command::new("open").arg(SAVE_DIR).spawn() {
        warn!("Failed to open config folder: {error}");
    }
}

pub fn config_path() -> PathBuf {
    PathBuf::from(SAVE_DIR).join(CONFIG_FILE)
}

pub fn key_from_input(keys: &ButtonInput<KeyCode>) -> Option<ConfigKey> {
    [
        ConfigKey::Escape,
        ConfigKey::Space,
        ConfigKey::ShiftLeft,
        ConfigKey::ShiftRight,
        ConfigKey::Slash,
        ConfigKey::KeyA,
        ConfigKey::KeyD,
        ConfigKey::KeyE,
        ConfigKey::KeyF,
        ConfigKey::KeyI,
        ConfigKey::KeyR,
        ConfigKey::KeyS,
        ConfigKey::KeyW,
        ConfigKey::Digit1,
        ConfigKey::Digit2,
        ConfigKey::Digit3,
        ConfigKey::Digit4,
        ConfigKey::Digit5,
        ConfigKey::Digit6,
        ConfigKey::Digit7,
        ConfigKey::Digit8,
        ConfigKey::Digit9,
    ]
    .into_iter()
    .find(|key| keys.just_pressed(key.key_code()))
}
