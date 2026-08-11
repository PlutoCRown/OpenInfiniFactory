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
    virtual_controls_opacity: 1.0,
    master_volume: 1.0,
    music_volume: 0.0,
    sfx_volume: 1.0,
    shadows_enabled: true,
    ssao_quality: ConfigSsaoQuality::High,
    vsync_enabled: true,
    skybox_enabled: true,
    window_mode: ConfigWindowMode::Windowed,
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
    #[serde(default = "default_virtual_controls_opacity")]
    pub virtual_controls_opacity: f32,
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,
    #[serde(default = "default_music_volume")]
    pub music_volume: f32,
    #[serde(default = "default_sfx_volume")]
    pub sfx_volume: f32,
    #[serde(default = "default_shadows_enabled")]
    pub shadows_enabled: bool,
    #[serde(default = "default_ssao_quality")]
    pub ssao_quality: ConfigSsaoQuality,
    #[serde(default = "default_vsync_enabled")]
    pub vsync_enabled: bool,
    #[serde(default = "default_skybox_enabled")]
    pub skybox_enabled: bool,
    /// 桌面窗口模式（Web/移动端忽略）
    #[serde(default)]
    pub window_mode: ConfigWindowMode,
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

/// 旧配置缺失触控控件透明度时使用默认值
fn default_virtual_controls_opacity() -> f32 {
    DEFAULT_CONFIG.virtual_controls_opacity
}

fn default_master_volume() -> f32 {
    DEFAULT_CONFIG.master_volume
}

fn default_music_volume() -> f32 {
    DEFAULT_CONFIG.music_volume
}

fn default_sfx_volume() -> f32 {
    DEFAULT_CONFIG.sfx_volume
}

fn default_shadows_enabled() -> bool {
    DEFAULT_CONFIG.shadows_enabled
}

fn default_ssao_quality() -> ConfigSsaoQuality {
    DEFAULT_CONFIG.ssao_quality
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
