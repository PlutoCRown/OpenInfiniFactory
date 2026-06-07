use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::shared::i18n::Language;
use crate::shared::save::SAVE_DIR;

pub const CONFIG_FILE: &str = "config.ron";

pub const DEFAULT_KEY_BINDINGS: KeyBindings = KeyBindings {
    pause: ConfigKey::Escape,
    inventory: ConfigKey::KeyE,
    alternate: ConfigKey::KeyC,
    rotate_or_rollback: ConfigKey::KeyR,
    simulate: ConfigKey::KeyF,
    simulation_step: ConfigKey::KeyC,
    simulation_fast: ConfigKey::KeyF,
    simulation_rollback: ConfigKey::KeyR,
    debug: ConfigKey::Slash,
    debug_structure: ConfigKey::KeyP,
    forward: ConfigKey::KeyW,
    backward: ConfigKey::KeyS,
    left: ConfigKey::KeyA,
    right: ConfigKey::KeyD,
    jump_or_fly_up: ConfigKey::Space,
    fly_down: ConfigKey::ShiftLeft,
    place: ConfigInput::MouseLeft,
    delete: ConfigInput::MouseRight,
    pick: ConfigInput::MouseMiddle,
};

pub const DEFAULT_CONFIG: GameConfig = GameConfig {
    fov_degrees: 70.0,
    ui_scale: 1.0,
    gravity_scale: 1.2,
    language: None,
    place_selection_mode: ConfigSelectionMode::Point,
    delete_selection_mode: ConfigSelectionMode::Point,
    key_bindings: DEFAULT_KEY_BINDINGS,
};

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub fov_degrees: f32,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default = "default_gravity_scale")]
    pub gravity_scale: f32,
    #[serde(default)]
    pub language: Option<Language>,
    #[serde(default)]
    pub place_selection_mode: ConfigSelectionMode,
    #[serde(default)]
    pub delete_selection_mode: ConfigSelectionMode,
    #[serde(default = "default_key_bindings")]
    pub key_bindings: KeyBindings,
}

impl Default for GameConfig {
    fn default() -> Self {
        DEFAULT_CONFIG
    }
}

fn default_ui_scale() -> f32 {
    DEFAULT_CONFIG.ui_scale
}

fn default_gravity_scale() -> f32 {
    DEFAULT_CONFIG.gravity_scale
}

fn default_key_bindings() -> KeyBindings {
    DEFAULT_CONFIG.key_bindings
}

fn default_debug_structure_key() -> ConfigKey {
    ConfigKey::KeyP
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub enum ConfigSelectionMode {
    #[default]
    Point,
    Line,
    Plane,
}

impl ConfigSelectionMode {
    pub const ALL: [ConfigSelectionMode; 3] = [
        ConfigSelectionMode::Point,
        ConfigSelectionMode::Line,
        ConfigSelectionMode::Plane,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Point => "selection_mode.point",
            Self::Line => "selection_mode.line",
            Self::Plane => "selection_mode.plane",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub pause: ConfigKey,
    pub inventory: ConfigKey,
    pub alternate: ConfigKey,
    pub rotate_or_rollback: ConfigKey,
    pub simulate: ConfigKey,
    pub simulation_step: ConfigKey,
    pub simulation_fast: ConfigKey,
    pub simulation_rollback: ConfigKey,
    pub debug: ConfigKey,
    #[serde(default = "default_debug_structure_key")]
    pub debug_structure: ConfigKey,
    pub forward: ConfigKey,
    pub backward: ConfigKey,
    pub left: ConfigKey,
    pub right: ConfigKey,
    pub jump_or_fly_up: ConfigKey,
    pub fly_down: ConfigKey,
    pub place: ConfigInput,
    pub delete: ConfigInput,
    pub pick: ConfigInput,
}

impl Default for KeyBindings {
    fn default() -> Self {
        DEFAULT_KEY_BINDINGS
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActionKeyName {
    /// 暂停游戏
    Pause,
    /// 打开背包
    Inventory,
    /// 替换方块
    Alternate,
    RotateOrRollback,
    /// 开始模拟
    Simulate,
    /// 单步模拟
    SimulationStep,
    /// 快速模拟
    SimulationFast,
    /// 回滚模拟
    SimulationRollback,
    /// 调试面板
    Debug,
    /// 调试结构
    DebugStructure,
    /// 向前移动
    Forward,
    /// 向后移动
    Backward,
    /// 向左移动
    Left,
    /// 向右移动
    Right,
    JumpOrFlyUp,
    /// 向下飞行
    FlyDown,
    /// 放置方块
    Place,
    /// 删除方块
    Delete,
    /// 拾取方块
    Pick,
}

impl ActionKeyName {
    pub const GENERAL: [ActionKeyName; 15] = [
        ActionKeyName::Pause,
        ActionKeyName::Inventory,
        ActionKeyName::Alternate,
        ActionKeyName::RotateOrRollback,
        ActionKeyName::Debug,
        ActionKeyName::DebugStructure,
        ActionKeyName::Forward,
        ActionKeyName::Backward,
        ActionKeyName::Left,
        ActionKeyName::Right,
        ActionKeyName::JumpOrFlyUp,
        ActionKeyName::FlyDown,
        ActionKeyName::Place,
        ActionKeyName::Delete,
        ActionKeyName::Pick,
    ];

    pub const SIMULATION: [ActionKeyName; 4] = [
        ActionKeyName::Simulate,
        ActionKeyName::SimulationStep,
        ActionKeyName::SimulationFast,
        ActionKeyName::SimulationRollback,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            ActionKeyName::Pause => "action.pause",
            ActionKeyName::Inventory => "action.inventory",
            ActionKeyName::Alternate => "action.alternate",
            ActionKeyName::RotateOrRollback => "action.rotate",
            ActionKeyName::Simulate => "action.simulation_start",
            ActionKeyName::SimulationStep => "action.simulation_step",
            ActionKeyName::SimulationFast => "action.simulation_fast",
            ActionKeyName::SimulationRollback => "action.simulation_rollback",
            ActionKeyName::Debug => "action.debug",
            ActionKeyName::DebugStructure => "action.debug_structure",
            ActionKeyName::Forward => "action.forward",
            ActionKeyName::Backward => "action.backward",
            ActionKeyName::Left => "action.left",
            ActionKeyName::Right => "action.right",
            ActionKeyName::JumpOrFlyUp => "action.jump_or_fly_up",
            ActionKeyName::FlyDown => "action.fly_down",
            ActionKeyName::Place => "action.place",
            ActionKeyName::Delete => "action.delete",
            ActionKeyName::Pick => "action.pick",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConfigInput {
    Key(ConfigKey),
    MouseLeft,
    MouseRight,
    MouseMiddle,
}

impl ConfigInput {
    pub fn key_code(self) -> Option<KeyCode> {
        match self {
            Self::Key(key) => Some(key.key_code()),
            Self::MouseLeft | Self::MouseRight | Self::MouseMiddle => None,
        }
    }

    pub fn mouse_button(self) -> Option<MouseButton> {
        match self {
            Self::MouseLeft => Some(MouseButton::Left),
            Self::MouseRight => Some(MouseButton::Right),
            Self::MouseMiddle => Some(MouseButton::Middle),
            Self::Key(_) => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Key(key) => key.name(),
            Self::MouseLeft => "Mouse Left",
            Self::MouseRight => "Mouse Right",
            Self::MouseMiddle => "Mouse Middle",
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
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyI,
    KeyP,
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
            ConfigKey::KeyC => KeyCode::KeyC,
            ConfigKey::KeyD => KeyCode::KeyD,
            ConfigKey::KeyE => KeyCode::KeyE,
            ConfigKey::KeyF => KeyCode::KeyF,
            ConfigKey::KeyI => KeyCode::KeyI,
            ConfigKey::KeyP => KeyCode::KeyP,
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
            ConfigKey::KeyC => "C",
            ConfigKey::KeyD => "D",
            ConfigKey::KeyE => "E",
            ConfigKey::KeyF => "F",
            ConfigKey::KeyI => "I",
            ConfigKey::KeyP => "P",
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
    pub fn key(&self, action: ActionKeyName) -> ConfigKey {
        match action {
            ActionKeyName::Pause => self.key_bindings.pause,
            ActionKeyName::Inventory => self.key_bindings.inventory,
            ActionKeyName::Alternate => self.key_bindings.alternate,
            ActionKeyName::RotateOrRollback => self.key_bindings.rotate_or_rollback,
            ActionKeyName::Simulate => self.key_bindings.simulate,
            ActionKeyName::SimulationStep => self.key_bindings.simulation_step,
            ActionKeyName::SimulationFast => self.key_bindings.simulation_fast,
            ActionKeyName::SimulationRollback => self.key_bindings.simulation_rollback,
            ActionKeyName::Debug => self.key_bindings.debug,
            ActionKeyName::DebugStructure => self.key_bindings.debug_structure,
            ActionKeyName::Forward => self.key_bindings.forward,
            ActionKeyName::Backward => self.key_bindings.backward,
            ActionKeyName::Left => self.key_bindings.left,
            ActionKeyName::Right => self.key_bindings.right,
            ActionKeyName::JumpOrFlyUp => self.key_bindings.jump_or_fly_up,
            ActionKeyName::FlyDown => self.key_bindings.fly_down,
            ActionKeyName::Place | ActionKeyName::Delete | ActionKeyName::Pick => {
                return self
                    .input(action)
                    .key_code()
                    .map(key_from_code)
                    .unwrap_or(ConfigKey::KeyI)
            }
        }
    }

    pub fn input(&self, action: ActionKeyName) -> ConfigInput {
        match action {
            ActionKeyName::Pause => ConfigInput::Key(self.key_bindings.pause),
            ActionKeyName::Inventory => ConfigInput::Key(self.key_bindings.inventory),
            ActionKeyName::Alternate => ConfigInput::Key(self.key_bindings.alternate),
            ActionKeyName::RotateOrRollback => {
                ConfigInput::Key(self.key_bindings.rotate_or_rollback)
            }
            ActionKeyName::Simulate => ConfigInput::Key(self.key_bindings.simulate),
            ActionKeyName::SimulationStep => ConfigInput::Key(self.key_bindings.simulation_step),
            ActionKeyName::SimulationFast => ConfigInput::Key(self.key_bindings.simulation_fast),
            ActionKeyName::SimulationRollback => {
                ConfigInput::Key(self.key_bindings.simulation_rollback)
            }
            ActionKeyName::Debug => ConfigInput::Key(self.key_bindings.debug),
            ActionKeyName::DebugStructure => ConfigInput::Key(self.key_bindings.debug_structure),
            ActionKeyName::Forward => ConfigInput::Key(self.key_bindings.forward),
            ActionKeyName::Backward => ConfigInput::Key(self.key_bindings.backward),
            ActionKeyName::Left => ConfigInput::Key(self.key_bindings.left),
            ActionKeyName::Right => ConfigInput::Key(self.key_bindings.right),
            ActionKeyName::JumpOrFlyUp => ConfigInput::Key(self.key_bindings.jump_or_fly_up),
            ActionKeyName::FlyDown => ConfigInput::Key(self.key_bindings.fly_down),
            ActionKeyName::Place => self.key_bindings.place,
            ActionKeyName::Delete => self.key_bindings.delete,
            ActionKeyName::Pick => self.key_bindings.pick,
        }
    }

    pub fn set_key(&mut self, action: ActionKeyName, key: ConfigKey) {
        match action {
            ActionKeyName::Pause => self.key_bindings.pause = key,
            ActionKeyName::Inventory => self.key_bindings.inventory = key,
            ActionKeyName::Alternate => self.key_bindings.alternate = key,
            ActionKeyName::RotateOrRollback => self.key_bindings.rotate_or_rollback = key,
            ActionKeyName::Simulate => self.key_bindings.simulate = key,
            ActionKeyName::SimulationStep => self.key_bindings.simulation_step = key,
            ActionKeyName::SimulationFast => self.key_bindings.simulation_fast = key,
            ActionKeyName::SimulationRollback => self.key_bindings.simulation_rollback = key,
            ActionKeyName::Debug => self.key_bindings.debug = key,
            ActionKeyName::DebugStructure => self.key_bindings.debug_structure = key,
            ActionKeyName::Forward => self.key_bindings.forward = key,
            ActionKeyName::Backward => self.key_bindings.backward = key,
            ActionKeyName::Left => self.key_bindings.left = key,
            ActionKeyName::Right => self.key_bindings.right = key,
            ActionKeyName::JumpOrFlyUp => self.key_bindings.jump_or_fly_up = key,
            ActionKeyName::FlyDown => self.key_bindings.fly_down = key,
            ActionKeyName::Place | ActionKeyName::Delete | ActionKeyName::Pick => {
                self.set_input(action, ConfigInput::Key(key));
            }
        }
    }

    pub fn set_input(&mut self, action: ActionKeyName, input: ConfigInput) {
        match action {
            ActionKeyName::Place => self.key_bindings.place = input,
            ActionKeyName::Delete => self.key_bindings.delete = input,
            ActionKeyName::Pick => self.key_bindings.pick = input,
            _ => {
                if let ConfigInput::Key(key) = input {
                    self.set_key(action, key);
                }
            }
        }
    }
}

fn key_from_code(key_code: KeyCode) -> ConfigKey {
    key_from_input_code(key_code).unwrap_or(ConfigKey::KeyI)
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
        ConfigKey::KeyC,
        ConfigKey::KeyD,
        ConfigKey::KeyE,
        ConfigKey::KeyF,
        ConfigKey::KeyI,
        ConfigKey::KeyP,
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

pub fn input_from_buttons(
    keys: &ButtonInput<KeyCode>,
    mouse_buttons: &ButtonInput<MouseButton>,
) -> Option<ConfigInput> {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        return Some(ConfigInput::MouseLeft);
    }
    if mouse_buttons.just_pressed(MouseButton::Right) {
        return Some(ConfigInput::MouseRight);
    }
    if mouse_buttons.just_pressed(MouseButton::Middle) {
        return Some(ConfigInput::MouseMiddle);
    }
    key_from_input(keys).map(ConfigInput::Key)
}

fn key_from_input_code(key_code: KeyCode) -> Option<ConfigKey> {
    [
        ConfigKey::Escape,
        ConfigKey::Space,
        ConfigKey::ShiftLeft,
        ConfigKey::ShiftRight,
        ConfigKey::Slash,
        ConfigKey::KeyA,
        ConfigKey::KeyC,
        ConfigKey::KeyD,
        ConfigKey::KeyE,
        ConfigKey::KeyF,
        ConfigKey::KeyI,
        ConfigKey::KeyP,
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
    .find(|key| key.key_code() == key_code)
}
