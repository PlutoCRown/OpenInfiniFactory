use bevy::prelude::*;

use crate::game::state::GameSettings;
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, MOUSE_SENSITIVITY_MAX, MOUSE_SENSITIVITY_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{ActionKeyName, ConfigSelectionMode};
use crate::shared::i18n::Language;

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
        control: SettingsControl::Dropdown(SettingsDropdown::PlaceSelectionMode),
    },
    SettingsItem {
        label_key: "settings.delete_selection_mode",
        control: SettingsControl::Dropdown(SettingsDropdown::DeleteSelectionMode),
    },
];

pub const GRAPHICS_SETTINGS: &[SettingsItem] = &[SettingsItem {
    label_key: "settings.shadows",
    control: SettingsControl::Dropdown(SettingsDropdown::Shadows),
}];

impl SettingsField {
    pub fn slider(self) -> Option<SettingsSliderConfig> {
        GAMEPLAY_SETTINGS
            .iter()
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

    pub fn display(self, settings: &GameSettings) -> String {
        use crate::game::ui::access::i18n;

        match self {
            Self::Fov => format!("FOV {:.0}", settings.fov_degrees),
            Self::UiScale => i18n.fmt(
                "settings.ui_scale",
                &[("scale", format!("{:.1}", settings.ui_scale))],
            ),
            Self::Gravity => i18n.fmt(
                "settings.gravity_value",
                &[("scale", format!("{:.1}", settings.gravity_scale))],
            ),
            Self::MouseSensitivityX => i18n.fmt(
                "settings.mouse_sensitivity_value",
                &[("scale", format!("{:.1}", settings.mouse_sensitivity_x))],
            ),
            Self::MouseSensitivityY => i18n.fmt(
                "settings.mouse_sensitivity_value",
                &[("scale", format!("{:.1}", settings.mouse_sensitivity_y))],
            ),
        }
    }

    pub fn apply_percent(
        self,
        percent: f32,
        settings: &mut GameSettings,
        ui_scale: &mut UiScale,
        config: &mut crate::shared::config::GameConfig,
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
        );
    }

    fn value(self, settings: &GameSettings) -> f32 {
        match self {
            Self::Fov => settings.fov_degrees,
            Self::UiScale => settings.ui_scale,
            Self::Gravity => settings.gravity_scale,
            Self::MouseSensitivityX => settings.mouse_sensitivity_x,
            Self::MouseSensitivityY => settings.mouse_sensitivity_y,
        }
    }

    fn apply_value(
        self,
        value: f32,
        settings: &mut GameSettings,
        ui_scale: &mut UiScale,
        config: &mut crate::shared::config::GameConfig,
    ) {
        match self {
            Self::Fov => {
                settings.fov_degrees = value;
                config.fov_degrees = value;
            }
            Self::UiScale => {
                settings.ui_scale = value;
                ui_scale.0 = value;
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
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsDropdown {
    Language,
    PlaceSelectionMode,
    DeleteSelectionMode,
    Shadows,
}

impl SettingsDropdown {
    pub fn trigger_label(self, config: &crate::shared::config::GameConfig) -> String {
        use crate::game::ui::access::i18n;

        match self {
            Self::Language => i18n.language().native_name().to_string(),
            Self::PlaceSelectionMode => i18n.t(config.place_selection_mode.label_key()),
            Self::DeleteSelectionMode => i18n.t(config.delete_selection_mode.label_key()),
            Self::Shadows => i18n.t(if config.shadows_enabled {
                "settings.option_on"
            } else {
                "settings.option_off"
            }),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsAction {
    TabGameplay,
    TabGraphics,
    TabKeyBindings,
    Field(SettingsField),
    SetPlaceSelectionMode(ConfigSelectionMode),
    SetDeleteSelectionMode(ConfigSelectionMode),
    SetLanguage(Language),
    SetShadowsEnabled(bool),
    ToggleDropdown(SettingsDropdown),
    Bind(ActionKeyName),
    ResetDefaults,
    OpenFolder,
    StartDebugHttp,
    Back,
}

impl UiActionLabel for SettingsAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::TabGameplay => "button.gameplay",
            Self::TabGraphics => "button.graphics",
            Self::TabKeyBindings => "button.key_bindings",
            Self::Bind(action) => action.label_key(),
            Self::ResetDefaults => "button.reset_defaults",
            Self::OpenFolder => "button.open_config_folder",
            Self::StartDebugHttp => "button.start_debug_http",
            Self::Back => "button.back",
            Self::Field(_)
            | Self::SetPlaceSelectionMode(_)
            | Self::SetDeleteSelectionMode(_)
            | Self::SetLanguage(_)
            | Self::SetShadowsEnabled(_)
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
        )
    }

    pub fn is_tab(self) -> bool {
        matches!(
            self,
            Self::TabGameplay | Self::TabGraphics | Self::TabKeyBindings
        )
    }
}

#[derive(Resource, Default)]
pub struct ActiveSettingsSlider(pub Option<SettingsField>);

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
pub enum SettingsTab {
    Gameplay,
    Graphics,
    KeyBindings,
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
