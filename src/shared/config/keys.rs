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

/// 屏幕空间环境光遮蔽（SSAO）质量档
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSsaoQuality {
    Off,
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

impl ConfigSsaoQuality {
    pub const ALL: [ConfigSsaoQuality; 5] = [
        ConfigSsaoQuality::Off,
        ConfigSsaoQuality::Low,
        ConfigSsaoQuality::Medium,
        ConfigSsaoQuality::High,
        ConfigSsaoQuality::Ultra,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Off => "settings.option_off",
            Self::Low => "settings.ssao.low",
            Self::Medium => "settings.ssao.medium",
            Self::High => "settings.ssao.high",
            Self::Ultra => "settings.ssao.ultra",
        }
    }
}

/// 桌面窗口显示模式
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigWindowMode {
    #[default]
    Windowed,
    /// 无边框（占满当前显示器）
    Borderless,
}

impl ConfigWindowMode {
    pub const ALL: [ConfigWindowMode; 2] =
        [ConfigWindowMode::Windowed, ConfigWindowMode::Borderless];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Windowed => "settings.window_mode.windowed",
            Self::Borderless => "settings.window_mode.borderless",
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
