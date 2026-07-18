use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::shared::i18n::Language;
use crate::shared::persistent_storage;

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
    undo: ConfigChord::default_undo(),
    redo: ConfigChord::default_redo(),
    copy: ConfigChord::default_copy(),
    paste: ConfigChord::default_paste(),
    toggle_selection_tool: ConfigChord::default_toggle_selection_tool(),
};

pub const DEFAULT_CONFIG: GameConfig = GameConfig {
    fov_degrees: 70.0,
    ui_scale: 1.0,
    gravity_scale: 1.2,
    mouse_sensitivity_x: 1.0,
    mouse_sensitivity_y: 1.0,
    shadows_enabled: true,
    vsync_enabled: true,
    skybox_enabled: false,
    language: None,
    place_selection_mode: ConfigSelectionMode::Point,
    delete_selection_mode: ConfigSelectionMode::Point,
    key_bindings: DEFAULT_KEY_BINDINGS,
    virtual_controls: VirtualControlsLayout::DEFAULT,
};

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub fov_degrees: f32,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default = "default_gravity_scale")]
    pub gravity_scale: f32,
    #[serde(default = "default_mouse_sensitivity")]
    pub mouse_sensitivity_x: f32,
    #[serde(default = "default_mouse_sensitivity")]
    pub mouse_sensitivity_y: f32,
    #[serde(default = "default_shadows_enabled")]
    pub shadows_enabled: bool,
    #[serde(default = "default_vsync_enabled")]
    pub vsync_enabled: bool,
    #[serde(default = "default_skybox_enabled")]
    pub skybox_enabled: bool,
    #[serde(default)]
    pub language: Option<Language>,
    #[serde(default)]
    pub place_selection_mode: ConfigSelectionMode,
    #[serde(default)]
    pub delete_selection_mode: ConfigSelectionMode,
    #[serde(default = "default_key_bindings")]
    pub key_bindings: KeyBindings,
    #[serde(default = "default_virtual_controls")]
    pub virtual_controls: VirtualControlsLayout,
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

fn default_mouse_sensitivity() -> f32 {
    DEFAULT_CONFIG.mouse_sensitivity_x
}

fn default_shadows_enabled() -> bool {
    DEFAULT_CONFIG.shadows_enabled
}

fn default_vsync_enabled() -> bool {
    DEFAULT_CONFIG.vsync_enabled
}

fn default_skybox_enabled() -> bool {
    DEFAULT_CONFIG.skybox_enabled
}

fn default_key_bindings() -> KeyBindings {
    DEFAULT_CONFIG.key_bindings
}

fn default_virtual_controls() -> VirtualControlsLayout {
    VirtualControlsLayout::DEFAULT
}

/// 虚拟遥感单个控件相对固定锚点的偏移与缩放
///
/// `offset_*` / 基准尺寸以 [`VIRTUAL_LAYOUT_REF_EDGE`] 为参考短边存档；
/// 运行时按 `min(宽, 高) / VIRTUAL_LAYOUT_REF_EDGE` 等比换算（横屏引导下短边即竖向边）。
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct VirtualControlTransform {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
}

/// 遥感布局存档参考短边（逻辑像素；移动端横屏时即屏幕高度）
pub const VIRTUAL_LAYOUT_REF_EDGE: f32 = 720.0;

impl VirtualControlTransform {
    pub const fn new(offset_x: f32, offset_y: f32, scale: f32) -> Self {
        Self {
            offset_x,
            offset_y,
            scale,
        }
    }

    pub const fn identity() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }
}

/// 虚拟遥感全部控件布局（锚点固定，仅存 offset/scale）
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VirtualControlsLayout {
    pub joystick: VirtualControlTransform,
    pub jump: VirtualControlTransform,
    pub place: VirtualControlTransform,
    pub delete: VirtualControlTransform,
    pub pause: VirtualControlTransform,
    pub simulate: VirtualControlTransform,
    pub sim_pause: VirtualControlTransform,
    pub sim_fast: VirtualControlTransform,
    pub sim_step: VirtualControlTransform,
    pub rotate: VirtualControlTransform,
    pub alternate: VirtualControlTransform,
    pub block_config: VirtualControlTransform,
    #[serde(default = "default_virtual_inventory")]
    pub inventory: VirtualControlTransform,
}

fn default_virtual_inventory() -> VirtualControlTransform {
    VirtualControlTransform::new(24.0, 282.0, 1.0)
}

impl VirtualControlsLayout {
    pub const DEFAULT: Self = Self {
        // 基于玩家手调布局校准：摇杆等距；右下三角同尺寸圆弧；右上/右侧齐边等距
        joystick: VirtualControlTransform::new(75.1875, 75.1875, 1.8123217),
        jump: VirtualControlTransform::new(41.4327, 242.987, 1.2227905),
        place: VirtualControlTransform::new(174.2844, 160.9637, 1.2227905),
        delete: VirtualControlTransform::new(247.7921, 23.2175, 1.2227905),
        pause: VirtualControlTransform::new(24.0, 24.0, 1.0),
        simulate: VirtualControlTransform::new(110.0, 24.0, 1.0),
        sim_pause: VirtualControlTransform::new(110.0, 24.0, 1.0),
        sim_fast: VirtualControlTransform::new(196.0, 24.0, 1.0),
        sim_step: VirtualControlTransform::new(282.0, 24.0, 1.0),
        rotate: VirtualControlTransform::new(24.0, 196.0, 1.0),
        alternate: VirtualControlTransform::new(24.0, 110.0, 1.0),
        block_config: VirtualControlTransform::new(58.625, 268.75, 1.0),
        inventory: VirtualControlTransform::new(24.0, 282.0, 1.0),
    };

    pub fn transform(&self, id: VirtualControlId) -> VirtualControlTransform {
        match id {
            VirtualControlId::Joystick => self.joystick,
            VirtualControlId::Jump => self.jump,
            VirtualControlId::Place => self.place,
            VirtualControlId::Delete => self.delete,
            VirtualControlId::Pause => self.pause,
            VirtualControlId::Simulate => self.simulate,
            VirtualControlId::SimPause => self.sim_pause,
            VirtualControlId::SimFast => self.sim_fast,
            VirtualControlId::SimStep => self.sim_step,
            VirtualControlId::Rotate => self.rotate,
            VirtualControlId::Alternate => self.alternate,
            VirtualControlId::BlockConfig => self.block_config,
            VirtualControlId::Inventory => self.inventory,
        }
    }

    pub fn set_transform(&mut self, id: VirtualControlId, transform: VirtualControlTransform) {
        match id {
            VirtualControlId::Joystick => self.joystick = transform,
            VirtualControlId::Jump => self.jump = transform,
            VirtualControlId::Place => self.place = transform,
            VirtualControlId::Delete => self.delete = transform,
            VirtualControlId::Pause => self.pause = transform,
            VirtualControlId::Simulate => self.simulate = transform,
            VirtualControlId::SimPause => self.sim_pause = transform,
            VirtualControlId::SimFast => self.sim_fast = transform,
            VirtualControlId::SimStep => self.sim_step = transform,
            VirtualControlId::Rotate => self.rotate = transform,
            VirtualControlId::Alternate => self.alternate = transform,
            VirtualControlId::BlockConfig => self.block_config = transform,
            VirtualControlId::Inventory => self.inventory = transform,
        }
    }
}

/// 虚拟遥感控件标识（锚点种类固定）
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VirtualControlId {
    Joystick,
    Jump,
    Place,
    Delete,
    Pause,
    Simulate,
    SimPause,
    SimFast,
    SimStep,
    Rotate,
    Alternate,
    BlockConfig,
    Inventory,
}

impl VirtualControlId {
    pub const ALL: &[Self] = &[
        Self::Joystick,
        Self::Jump,
        Self::Place,
        Self::Delete,
        Self::Pause,
        Self::Simulate,
        Self::SimPause,
        Self::SimFast,
        Self::SimStep,
        Self::Rotate,
        Self::Alternate,
        Self::BlockConfig,
        Self::Inventory,
    ];

    pub fn anchor(self) -> VirtualControlAnchor {
        match self {
            Self::Joystick => VirtualControlAnchor::BottomLeft,
            Self::Jump | Self::Place | Self::Delete => VirtualControlAnchor::BottomRight,
            Self::Pause | Self::Simulate | Self::SimPause | Self::SimFast | Self::SimStep => {
                VirtualControlAnchor::TopRight
            }
            Self::Rotate | Self::Alternate | Self::Inventory => {
                VirtualControlAnchor::TopRightColumn
            }
            Self::BlockConfig => VirtualControlAnchor::BottomCenter,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Joystick => "virtual.joystick",
            Self::Jump => "action.jump_or_fly_up",
            Self::Place => "action.place",
            Self::Delete => "action.delete",
            Self::Pause => "action.pause",
            Self::Simulate => "action.simulate",
            Self::SimPause => "virtual.sim_pause",
            Self::SimFast => "action.simulation_fast",
            Self::SimStep => "action.simulation_step",
            Self::Rotate => "action.rotate_or_rollback",
            Self::Alternate => "action.alternate",
            Self::BlockConfig => "virtual.block_config",
            Self::Inventory => "action.inventory",
        }
    }
}

/// 虚拟控件屏幕锚点（不可在设置中更改）
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirtualControlAnchor {
    BottomLeft,
    BottomRight,
    TopRight,
    TopRightColumn,
    BottomCenter,
}

fn default_debug_structure_key() -> ConfigKey {
    ConfigKey::KeyP
}

fn default_undo_chord() -> ConfigChord {
    ConfigChord::default_undo()
}

fn default_redo_chord() -> ConfigChord {
    ConfigChord::default_redo()
}

fn default_copy_chord() -> ConfigChord {
    ConfigChord::default_copy()
}

fn default_paste_chord() -> ConfigChord {
    ConfigChord::default_paste()
}

fn default_toggle_selection_tool_chord() -> ConfigChord {
    ConfigChord::default_toggle_selection_tool()
}

/// 组合键：修饰键 + 主键（primary_modifier 在 macOS 为 Command，其它平台为 Control）
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigChord {
    pub key: ConfigKey,
    pub primary_modifier: bool,
    pub shift: bool,
    pub alt: bool,
}

impl ConfigChord {
    pub const fn default_undo() -> Self {
        Self {
            key: ConfigKey::KeyZ,
            primary_modifier: true,
            shift: false,
            alt: false,
        }
    }

    pub const fn default_redo() -> Self {
        Self {
            key: ConfigKey::KeyZ,
            primary_modifier: true,
            shift: true,
            alt: false,
        }
    }

    pub const fn default_copy() -> Self {
        Self {
            key: ConfigKey::KeyC,
            primary_modifier: true,
            shift: false,
            alt: false,
        }
    }

    pub const fn default_paste() -> Self {
        Self {
            key: ConfigKey::KeyV,
            primary_modifier: true,
            shift: false,
            alt: false,
        }
    }

    pub const fn default_toggle_selection_tool() -> Self {
        Self {
            key: ConfigKey::KeyX,
            primary_modifier: true,
            shift: false,
            alt: false,
        }
    }

    pub fn display_name(self) -> String {
        let mut parts = Vec::new();
        if self.primary_modifier {
            parts.push(primary_modifier_label().to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        parts.push(self.key.name().to_string());
        parts.join("+")
    }

    pub fn just_triggered(self, keys: &ButtonInput<KeyCode>) -> bool {
        keys.just_pressed(self.key.key_code())
            && self.primary_modifier == primary_modifier_pressed(keys)
            && self.shift == shift_modifier_pressed(keys)
            && self.alt == alt_modifier_pressed(keys)
    }
}

/// macOS 为 Command，其它平台为 Control（不用 ⌘ 符号，UI 字体可能缺字）
pub fn primary_modifier_label() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Command"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl"
    }
}

pub fn primary_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    #[cfg(target_os = "macos")]
    {
        keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight)
    }
    #[cfg(not(target_os = "macos"))]
    {
        keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
    }
}

pub fn shift_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

pub fn alt_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight)
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
    #[serde(default = "default_undo_chord")]
    pub undo: ConfigChord,
    #[serde(default = "default_redo_chord")]
    pub redo: ConfigChord,
    #[serde(default = "default_copy_chord")]
    pub copy: ConfigChord,
    #[serde(default = "default_paste_chord")]
    pub paste: ConfigChord,
    #[serde(default = "default_toggle_selection_tool_chord")]
    pub toggle_selection_tool: ConfigChord,
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
    /// 撤销编辑
    Undo,
    /// 重做编辑
    Redo,
    /// 复制（选区拖动中立即复制放置；编辑模式为复制系统方块配置）
    Copy,
    /// 粘贴系统方块配置
    Paste,
    /// 切换选区工具
    ToggleSelectionTool,
}

impl ActionKeyName {
    pub const GENERAL: [ActionKeyName; 17] = [
        ActionKeyName::Pause,
        ActionKeyName::Inventory,
        ActionKeyName::Alternate,
        ActionKeyName::RotateOrRollback,
        ActionKeyName::Undo,
        ActionKeyName::Redo,
        ActionKeyName::Copy,
        ActionKeyName::Paste,
        ActionKeyName::ToggleSelectionTool,
        ActionKeyName::Debug,
        ActionKeyName::DebugStructure,
        ActionKeyName::Forward,
        ActionKeyName::Backward,
        ActionKeyName::Left,
        ActionKeyName::Right,
        ActionKeyName::JumpOrFlyUp,
        ActionKeyName::FlyDown,
    ];

    pub const MOUSE: [ActionKeyName; 3] = [
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

    pub fn is_chord(self) -> bool {
        matches!(
            self,
            Self::Undo | Self::Redo | Self::Copy | Self::Paste | Self::ToggleSelectionTool
        )
    }

    pub fn label_key(self) -> &'static str {
        match self {
            ActionKeyName::Pause => "action.pause",
            ActionKeyName::Inventory => "action.inventory",
            ActionKeyName::Alternate => "action.alternate",
            ActionKeyName::RotateOrRollback => "action.rotate",
            ActionKeyName::Undo => "action.undo",
            ActionKeyName::Redo => "action.redo",
            ActionKeyName::Copy => "action.copy",
            ActionKeyName::Paste => "action.paste",
            ActionKeyName::ToggleSelectionTool => "action.toggle_selection_tool",
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
    KeyV,
    KeyW,
    KeyX,
    KeyZ,
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
            ConfigKey::KeyV => KeyCode::KeyV,
            ConfigKey::KeyW => KeyCode::KeyW,
            ConfigKey::KeyX => KeyCode::KeyX,
            ConfigKey::KeyZ => KeyCode::KeyZ,
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

    pub fn is_modifier(self) -> bool {
        matches!(self, Self::ShiftLeft | Self::ShiftRight)
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
            ConfigKey::KeyV => "V",
            ConfigKey::KeyW => "W",
            ConfigKey::KeyX => "X",
            ConfigKey::KeyZ => "Z",
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
    pub fn chord(&self, action: ActionKeyName) -> ConfigChord {
        match action {
            ActionKeyName::Undo => self.key_bindings.undo,
            ActionKeyName::Redo => self.key_bindings.redo,
            ActionKeyName::Copy => self.key_bindings.copy,
            ActionKeyName::Paste => self.key_bindings.paste,
            ActionKeyName::ToggleSelectionTool => self.key_bindings.toggle_selection_tool,
            _ => ConfigChord::default_undo(),
        }
    }

    pub fn binding_display(&self, action: ActionKeyName) -> String {
        if action.is_chord() {
            self.chord(action).display_name()
        } else {
            self.input(action).name().to_string()
        }
    }

    pub fn set_chord(&mut self, action: ActionKeyName, chord: ConfigChord) {
        match action {
            ActionKeyName::Undo => self.key_bindings.undo = chord,
            ActionKeyName::Redo => self.key_bindings.redo = chord,
            ActionKeyName::Copy => self.key_bindings.copy = chord,
            ActionKeyName::Paste => self.key_bindings.paste = chord,
            ActionKeyName::ToggleSelectionTool => self.key_bindings.toggle_selection_tool = chord,
            _ => {}
        }
    }

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
            ActionKeyName::Undo => self.key_bindings.undo.key,
            ActionKeyName::Redo => self.key_bindings.redo.key,
            ActionKeyName::Copy => self.key_bindings.copy.key,
            ActionKeyName::Paste => self.key_bindings.paste.key,
            ActionKeyName::ToggleSelectionTool => self.key_bindings.toggle_selection_tool.key,
            ActionKeyName::Place | ActionKeyName::Delete | ActionKeyName::Pick => {
                return self
                    .input(action)
                    .key_code()
                    .map(key_from_code)
                    .unwrap_or(ConfigKey::KeyI);
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
            ActionKeyName::Undo
            | ActionKeyName::Redo
            | ActionKeyName::Copy
            | ActionKeyName::Paste
            | ActionKeyName::ToggleSelectionTool => ConfigInput::Key(self.chord(action).key),
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
            ActionKeyName::Undo
            | ActionKeyName::Redo
            | ActionKeyName::Copy
            | ActionKeyName::Paste
            | ActionKeyName::ToggleSelectionTool => {}
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
    let key = persistent_storage::config_key();
    let Some(contents) = persistent_storage::read(key) else {
        let config = GameConfig::default();
        // Web hydrate 前不要落盘，避免被 replace_all 冲掉
        if persistent_storage::is_ready() {
            save_config(&config);
        }
        return config;
    };
    ron::from_str::<GameConfig>(&contents).unwrap_or_else(|error| {
        warn!("Failed to load config: {error}");
        GameConfig::default()
    })
}

pub fn save_config(config: &GameConfig) {
    let key = persistent_storage::config_key();
    match ron::ser::to_string_pretty(config, PrettyConfig::default()) {
        Ok(serialized) => {
            if !persistent_storage::write(key, &serialized) {
                warn!("Failed to write config");
            }
        }
        Err(error) => warn!("Failed to serialize config: {error}"),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn open_config_folder() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_config_folder() {
    use std::fs;
    use std::process::Command;

    let dir = crate::shared::platform::saves_directory();
    if let Err(error) = fs::create_dir_all(dir) {
        warn!("Failed to create config directory: {error}");
        return;
    }

    if let Err(error) = Command::new("open").arg(dir).spawn() {
        warn!("Failed to open config folder: {error}");
    }
}

const CONFIG_KEYS: [ConfigKey; 27] = [
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
    ConfigKey::KeyV,
    ConfigKey::KeyW,
    ConfigKey::KeyX,
    ConfigKey::KeyZ,
    ConfigKey::Digit1,
    ConfigKey::Digit2,
    ConfigKey::Digit3,
    ConfigKey::Digit4,
    ConfigKey::Digit5,
    ConfigKey::Digit6,
    ConfigKey::Digit7,
    ConfigKey::Digit8,
    ConfigKey::Digit9,
];

pub fn chord_from_input(keys: &ButtonInput<KeyCode>) -> Option<ConfigChord> {
    let key = CONFIG_KEYS
        .iter()
        .copied()
        .filter(|key| !key.is_modifier())
        .find(|key| keys.just_pressed(key.key_code()))?;
    Some(ConfigChord {
        key,
        primary_modifier: primary_modifier_pressed(keys),
        shift: shift_modifier_pressed(keys),
        alt: alt_modifier_pressed(keys),
    })
}

pub fn key_from_input(keys: &ButtonInput<KeyCode>) -> Option<ConfigKey> {
    CONFIG_KEYS
        .iter()
        .copied()
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
    CONFIG_KEYS
        .iter()
        .copied()
        .find(|key| key.key_code() == key_code)
}
