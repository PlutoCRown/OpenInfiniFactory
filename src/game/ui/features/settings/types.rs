use bevy::prelude::*;

use crate::game::state::GameSettings;
use crate::game::{
    GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, MOUSE_SENSITIVITY_MAX, MOUSE_SENSITIVITY_MIN,
    UI_SCALE_MAX, UI_SCALE_MIN,
};
use crate::shared::config::{
    ActionKeyName, ConfigGameplayRenderRate, ConfigSelectionMode, ConfigWindowMode,
};
use crate::shared::i18n::Language;
use crate::shared::touch_profile::TouchProfile;

use crate::game::ui::core::action::UiActionLabel;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsText(pub SettingsTextKind);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsTextKind {
    KeyBinding,
    DebugHttp,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsValueText(pub SettingsField);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsSliderFill(pub SettingsField);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsSliderKnob(pub SettingsField);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownLabel(pub SettingsDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownList(pub SettingsDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownRow(pub SettingsDropdown);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsField {
    Fov,
    UiScale,
    Gravity,
    MouseSensitivityX,
    MouseSensitivityY,
    VirtualControlsOpacity,
    MasterVolume,
    MusicVolume,
    SfxVolume,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsSliderTrigger {
    Live,
    Commit,
}

#[derive(Clone, Copy)]
pub struct SettingsSliderConfig {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub trigger: SettingsSliderTrigger,
}

#[derive(Clone, Copy)]
pub enum SettingsControl {
    Slider {
        field: SettingsField,
        config: SettingsSliderConfig,
    },
    Dropdown(SettingsDropdown),
    /// 横向单选按钮组（选项与同名 Dropdown 一致）
    Radio(SettingsDropdown),
}

#[derive(Clone, Copy)]
pub struct SettingsItem {
    pub label_key: &'static str,
    pub control: SettingsControl,
}

pub const GAMEPLAY_SETTINGS: &[SettingsItem] = &[
    SettingsItem {
        label_key: "settings.fov",
        control: SettingsControl::Slider {
            field: SettingsField::Fov,
            config: SettingsSliderConfig {
                min: 50.0,
                max: 110.0,
                step: 1.0,
                trigger: SettingsSliderTrigger::Live,
            },
        },
    },
    SettingsItem {
        label_key: "settings.ui_scale_label",
        control: SettingsControl::Slider {
            field: SettingsField::UiScale,
            config: SettingsSliderConfig {
                min: UI_SCALE_MIN,
                max: UI_SCALE_MAX,
                step: 0.1,
                trigger: SettingsSliderTrigger::Commit,
            },
        },
    },
    SettingsItem {
        label_key: "settings.gravity",
        control: SettingsControl::Slider {
            field: SettingsField::Gravity,
            config: SettingsSliderConfig {
                min: GRAVITY_SCALE_MIN,
                max: GRAVITY_SCALE_MAX,
                step: 0.1,
                trigger: SettingsSliderTrigger::Commit,
            },
        },
    },
    SettingsItem {
        label_key: "settings.mouse_sensitivity_x",
        control: SettingsControl::Slider {
            field: SettingsField::MouseSensitivityX,
            config: SettingsSliderConfig {
                min: MOUSE_SENSITIVITY_MIN,
                max: MOUSE_SENSITIVITY_MAX,
                step: 0.1,
                trigger: SettingsSliderTrigger::Live,
            },
        },
    },
    SettingsItem {
        label_key: "settings.mouse_sensitivity_y",
        control: SettingsControl::Slider {
            field: SettingsField::MouseSensitivityY,
            config: SettingsSliderConfig {
                min: MOUSE_SENSITIVITY_MIN,
                max: MOUSE_SENSITIVITY_MAX,
                step: 0.1,
                trigger: SettingsSliderTrigger::Live,
            },
        },
    },
    SettingsItem {
        label_key: "settings.language",
        control: SettingsControl::Dropdown(SettingsDropdown::Language),
    },
    SettingsItem {
        label_key: "settings.place_selection_mode",
        control: SettingsControl::Radio(SettingsDropdown::PlaceSelectionMode),
    },
    SettingsItem {
        label_key: "settings.delete_selection_mode",
        control: SettingsControl::Radio(SettingsDropdown::DeleteSelectionMode),
    },
];

pub const GRAPHICS_SETTINGS: &[SettingsItem] = &[
    SettingsItem {
        label_key: "settings.shadows",
        control: SettingsControl::Radio(SettingsDropdown::Shadows),
    },
    SettingsItem {
        label_key: "settings.ssao",
        control: SettingsControl::Dropdown(SettingsDropdown::Ssao),
    },
    SettingsItem {
        label_key: "settings.render_rate",
        control: SettingsControl::Dropdown(SettingsDropdown::GameplayRenderRate),
    },
    SettingsItem {
        label_key: "settings.vsync",
        control: SettingsControl::Radio(SettingsDropdown::Vsync),
    },
    SettingsItem {
        label_key: "settings.skybox",
        control: SettingsControl::Radio(SettingsDropdown::Skybox),
    },
    SettingsItem {
        label_key: "settings.window_mode",
        control: SettingsControl::Dropdown(SettingsDropdown::WindowMode),
    },
];

pub const AUDIO_SETTINGS: &[SettingsItem] = &[
    SettingsItem {
        label_key: "settings.master_volume",
        control: SettingsControl::Slider {
            field: SettingsField::MasterVolume,
            config: SettingsSliderConfig {
                min: 0.0,
                max: 1.0,
                step: 0.05,
                trigger: SettingsSliderTrigger::Live,
            },
        },
    },
    SettingsItem {
        label_key: "settings.music_volume",
        control: SettingsControl::Slider {
            field: SettingsField::MusicVolume,
            config: SettingsSliderConfig {
                min: 0.0,
                max: 1.0,
                step: 0.05,
                trigger: SettingsSliderTrigger::Live,
            },
        },
    },
    SettingsItem {
        label_key: "settings.sfx_volume",
        control: SettingsControl::Slider {
            field: SettingsField::SfxVolume,
            config: SettingsSliderConfig {
                min: 0.0,
                max: 1.0,
                step: 0.05,
                trigger: SettingsSliderTrigger::Live,
            },
        },
    },
];

/// 仅在触控模式显示的设置项
pub const TOUCH_SETTINGS: &[SettingsItem] = &[SettingsItem {
    label_key: "settings.virtual_controls_opacity",
    control: SettingsControl::Slider {
        field: SettingsField::VirtualControlsOpacity,
        config: SettingsSliderConfig {
            min: 0.0,
            max: 1.0,
            step: 0.05,
            trigger: SettingsSliderTrigger::Live,
        },
    },
}];

impl SettingsField {
    pub fn slider(self) -> Option<SettingsSliderConfig> {
        GAMEPLAY_SETTINGS
            .iter()
            .chain(AUDIO_SETTINGS)
            .chain(TOUCH_SETTINGS)
            .find_map(|item| match item.control {
                SettingsControl::Slider { field, config } if field == self => Some(config),
                _ => None,
            })
    }

    pub fn percent(self, settings: &GameSettings) -> f32 {
        let Some(slider) = self.slider() else {
            return 0.0;
        };
        ((self.value(settings) - slider.min) / (slider.max - slider.min) * 100.0).clamp(0.0, 100.0)
    }

    pub fn display(self, settings: &GameSettings, preview: Option<f32>) -> String {
        use crate::game::ui::access::i18n;

        let value = preview
            .and_then(|percent| {
                self.slider().map(|slider| {
                    let raw = slider.min + percent.clamp(0.0, 1.0) * (slider.max - slider.min);
                    ((raw / slider.step).round() * slider.step).clamp(slider.min, slider.max)
                })
            })
            .unwrap_or_else(|| self.value(settings));
        match self {
            Self::Fov => format!("FOV {value:.0}"),
            Self::UiScale => {
                let scale = format!("{value:.1}");
                i18n.fmt("settings.ui_scale", &[("scale", scale.as_str())])
            }
            Self::Gravity => {
                let scale = format!("{value:.1}");
                i18n.fmt("settings.gravity_value", &[("scale", scale.as_str())])
            }
            Self::MouseSensitivityX => {
                let scale = format!("{value:.1}");
                i18n.fmt(
                    "settings.mouse_sensitivity_value",
                    &[("scale", scale.as_str())],
                )
            }
            Self::MouseSensitivityY => {
                let scale = format!("{value:.1}");
                i18n.fmt(
                    "settings.mouse_sensitivity_value",
                    &[("scale", scale.as_str())],
                )
            }
            Self::VirtualControlsOpacity => format!("{value:.2}"),
            Self::MasterVolume | Self::MusicVolume | Self::SfxVolume => {
                format!("{:.0}%", value * 100.0)
            }
        }
    }

    pub fn apply_percent(
        self,
        percent: f32,
        settings: &mut GameSettings,
        ui_scale: &mut UiScale,
        config: &mut crate::shared::config::GameConfig,
        touch: TouchProfile,
    ) {
        let Some(slider) = self.slider() else {
            return;
        };
        let raw = slider.min + percent.clamp(0.0, 1.0) * (slider.max - slider.min);
        let value = (raw / slider.step).round() * slider.step;
        self.apply_value(
            value.clamp(slider.min, slider.max),
            settings,
            ui_scale,
            config,
            touch,
        );
    }

    fn value(self, settings: &GameSettings) -> f32 {
        match self {
            Self::Fov => settings.fov_degrees,
            Self::UiScale => settings.ui_scale,
            Self::Gravity => settings.gravity_scale,
            Self::MouseSensitivityX => settings.mouse_sensitivity_x,
            Self::MouseSensitivityY => settings.mouse_sensitivity_y,
            Self::VirtualControlsOpacity => settings.virtual_controls_opacity,
            Self::MasterVolume => settings.master_volume,
            Self::MusicVolume => settings.music_volume,
            Self::SfxVolume => settings.sfx_volume,
        }
    }

    fn apply_value(
        self,
        value: f32,
        settings: &mut GameSettings,
        ui_scale: &mut UiScale,
        config: &mut crate::shared::config::GameConfig,
        touch: TouchProfile,
    ) {
        match self {
            Self::Fov => {
                settings.fov_degrees = value;
                config.fov_degrees = value;
            }
            Self::UiScale => {
                settings.ui_scale = value;
                ui_scale.0 = touch.effective_ui_scale(value);
                config.ui_scale = value;
            }
            Self::Gravity => {
                settings.gravity_scale = value;
                config.gravity_scale = value;
            }
            Self::MouseSensitivityX => {
                settings.mouse_sensitivity_x = value;
                config.mouse_sensitivity_x = value;
            }
            Self::MouseSensitivityY => {
                settings.mouse_sensitivity_y = value;
                config.mouse_sensitivity_y = value;
            }
            Self::VirtualControlsOpacity => {
                settings.virtual_controls_opacity = value;
                config.virtual_controls_opacity = value;
            }
            Self::MasterVolume => {
                settings.master_volume = value;
                config.master_volume = value;
            }
            Self::MusicVolume => {
                settings.music_volume = value;
                config.music_volume = value;
            }
            Self::SfxVolume => {
                settings.sfx_volume = value;
                config.sfx_volume = value;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsDropdown {
    Language,
    PlaceSelectionMode,
    DeleteSelectionMode,
    Shadows,
    Ssao,
    GameplayRenderRate,
    Vsync,
    Skybox,
    WindowMode,
}

impl SettingsDropdown {
    pub fn trigger_label(self, config: &crate::shared::config::GameConfig) -> String {
        use crate::game::ui::access::i18n;

        match self {
            Self::Language => {
                let pending = config.language.unwrap_or_else(|| i18n.language());
                let active = i18n.language();
                if pending != active {
                    i18n.fmt(
                        "settings.language_pending_restart",
                        &[("lang", pending.native_name())],
                    )
                } else {
                    pending.native_name().to_string()
                }
            }
            Self::PlaceSelectionMode => i18n.t(config.place_selection_mode.label_key()),
            Self::DeleteSelectionMode => i18n.t(config.delete_selection_mode.label_key()),
            Self::Shadows => i18n.t(if config.shadows_enabled {
                "settings.option_on"
            } else {
                "settings.option_off"
            }),
            Self::Ssao => i18n.t(config.ssao_quality.label_key()),
            Self::GameplayRenderRate => i18n.t(config.gameplay_render_rate.label_key()),
            Self::Vsync => i18n.t(if config.vsync_enabled {
                "settings.option_on"
            } else {
                "settings.option_off"
            }),
            Self::Skybox => i18n.t(if config.skybox_enabled {
                "settings.option_on"
            } else {
                "settings.option_off"
            }),
            Self::WindowMode => i18n.t(config.window_mode.label_key()),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsAction {
    TabGameplay,
    TabGraphics,
    TabKeyBindings,
    TabAudio,
    Field(SettingsField),
    SetPlaceSelectionMode(ConfigSelectionMode),
    SetDeleteSelectionMode(ConfigSelectionMode),
    SetLanguage(Language),
    SetShadowsEnabled(bool),
    SetSsaoQuality(crate::shared::config::ConfigSsaoQuality),
    SetGameplayRenderRate(ConfigGameplayRenderRate),
    SetVsyncEnabled(bool),
    SetSkyboxEnabled(bool),
    SetWindowMode(ConfigWindowMode),
    ToggleDropdown(SettingsDropdown),
    Bind(ActionKeyName),
    OpenVirtualLayout,
    ResetDefaults,
    OpenFolder,
    StartDebugHttp,
}

impl UiActionLabel for SettingsAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::TabGameplay => "button.gameplay",
            Self::TabGraphics => "button.graphics",
            Self::TabKeyBindings => "button.key_bindings",
            Self::TabAudio => "button.audio",
            Self::Bind(action) => action.label_key(),
            Self::OpenVirtualLayout => "virtual.layout_open",
            Self::ResetDefaults => "button.reset_defaults",
            Self::OpenFolder => "button.open_config_folder",
            Self::StartDebugHttp => "button.start_debug_http",
            Self::Field(_)
            | Self::SetPlaceSelectionMode(_)
            | Self::SetDeleteSelectionMode(_)
            | Self::SetLanguage(_)
            | Self::SetShadowsEnabled(_)
            | Self::SetSsaoQuality(_)
            | Self::SetGameplayRenderRate(_)
            | Self::SetVsyncEnabled(_)
            | Self::SetSkyboxEnabled(_)
            | Self::SetWindowMode(_)
            | Self::ToggleDropdown(_) => "",
        }
    }
}

impl SettingsAction {
    pub fn tab_selected(self, tab: SettingsTab) -> bool {
        matches!(
            (self, tab),
            (Self::TabGameplay, SettingsTab::Gameplay)
                | (Self::TabGraphics, SettingsTab::Graphics)
                | (Self::TabKeyBindings, SettingsTab::KeyBindings)
                | (Self::TabAudio, SettingsTab::Audio)
        )
    }

    /// 单选选项是否对应当前配置
    pub fn radio_selected(self, config: &crate::shared::config::GameConfig) -> bool {
        match self {
            Self::SetPlaceSelectionMode(mode) => config.place_selection_mode == mode,
            Self::SetDeleteSelectionMode(mode) => config.delete_selection_mode == mode,
            Self::SetShadowsEnabled(enabled) => config.shadows_enabled == enabled,
            Self::SetVsyncEnabled(enabled) => config.vsync_enabled == enabled,
            Self::SetSkyboxEnabled(enabled) => config.skybox_enabled == enabled,
            _ => false,
        }
    }

    pub fn is_tab(self) -> bool {
        matches!(
            self,
            Self::TabGameplay | Self::TabGraphics | Self::TabKeyBindings | Self::TabAudio
        )
    }
}

/// 设置滑条正在交互的预览值；提交型滑条在松手前不修改游戏设置
#[derive(Resource, Default)]
pub struct ActiveSettingsSlider {
    pub field: Option<SettingsField>,
    pub percent: f32,
}

impl ActiveSettingsSlider {
    pub fn preview(&self, field: SettingsField) -> Option<f32> {
        (self.field == Some(field)).then_some(self.percent)
    }

    pub fn clear(&mut self) {
        self.field = None;
    }
}

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
pub enum SettingsTab {
    Gameplay,
    Graphics,
    KeyBindings,
    Audio,
}

impl Default for SettingsTab {
    fn default() -> Self {
        Self::Gameplay
    }
}

#[derive(Resource, Default)]
pub struct PendingKeyBind(pub Option<ActionKeyName>);

#[derive(Resource, Default)]
pub struct OpenSettingsDropdown(pub Option<SettingsDropdown>);

/// 设置页自己的可见性标记，通用面板不理解设置标签。
#[derive(Component)]
pub struct SettingsTabPanel(pub SettingsTab);
