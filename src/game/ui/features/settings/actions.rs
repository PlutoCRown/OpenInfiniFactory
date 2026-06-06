use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::ui_widgets::{CoreSliderDragState, Slider, SliderRange, SliderValue};

use crate::game::state::{GameSettings, UiPanelId};
use crate::game::ui::access::{i18n, ui, UiMainThread};
use crate::game::ui::core::host::{UiAction, UiActionKind, UiHost, UiInstanceId};
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{input_from_buttons, open_config_folder, save_config, GameConfig};
use crate::shared::i18n::resolve_language;

use super::types::{
    ActiveSettingsSlider, OpenSettingsDropdown, PendingKeyBind, SettingsAction,
    SettingsSliderTrigger, SettingsTab,
};

pub fn settings_menu_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    runtime: Res<UiRuntime>,
    slider_values: Query<(&SettingsAction, &SliderValue, &SliderRange), With<Slider>>,
    slider_changes: Query<
        (
            &SettingsAction,
            Ref<SliderValue>,
            &SliderRange,
            &CoreSliderDragState,
        ),
        (
            With<Slider>,
            Or<(Changed<SliderValue>, Changed<CoreSliderDragState>)>,
        ),
    >,
) {
    if !runtime.is_settings_open() {
        pending_key_bind.0 = None;
        open_dropdown.0 = None;
        active_slider.0 = None;
        return;
    }

    if let Some(action) = pending_key_bind.0 {
        if let Some(input) = input_from_buttons(&keys, &mouse_buttons) {
            config.set_input(action, input);
            save_config(&config);
            pending_key_bind.0 = None;
        }
    }

    update_settings_sliders_from_input(
        &slider_changes,
        &mut active_slider,
        &mut settings,
        &mut ui_scale,
        &mut config,
    );

    if mouse_buttons.just_released(MouseButton::Left) {
        commit_active_settings_slider(
            &slider_values,
            &mut active_slider,
            &mut settings,
            &mut ui_scale,
            &mut config,
        );
    }
}

pub fn emit_settings_actions(
    mut click: On<Pointer<Click>>,
    ui_host: Res<UiHost>,
    runtime: Res<UiRuntime>,
    mut writer: MessageWriter<UiAction>,
    actions: Query<&SettingsAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) || !runtime.is_settings_open() {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    writer.write(UiAction {
        instance: UiInstanceId::SETTINGS,
        kind: UiActionKind::Settings(action),
    });
}

pub fn dispatch_settings_actions(
    _ui_thread: UiMainThread,
    mut actions: MessageReader<UiAction>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    mut settings_tab: ResMut<SettingsTab>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    mut commands: Commands,
) {
    for action in actions.read() {
        if action.instance != UiInstanceId::SETTINGS {
            continue;
        }
        let UiActionKind::Settings(action) = action.kind.clone() else {
            continue;
        };
        match action {
            SettingsAction::TabGameplay => {
                *settings_tab = SettingsTab::Gameplay;
                open_dropdown.0 = None;
            }
            SettingsAction::TabKeyBindings => {
                *settings_tab = SettingsTab::KeyBindings;
                open_dropdown.0 = None;
            }
            SettingsAction::Field(field) => {
                active_slider.0 = Some(field);
            }
            SettingsAction::SetPlaceSelectionMode(selection_mode) => {
                config.place_selection_mode = selection_mode;
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetDeleteSelectionMode(selection_mode) => {
                config.delete_selection_mode = selection_mode;
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetLanguage(language) => {
                i18n.set_language(language);
                config.language = Some(language);
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::ToggleDropdown(dropdown) => {
                open_dropdown.0 = if open_dropdown.0 == Some(dropdown) {
                    None
                } else {
                    Some(dropdown)
                };
            }
            SettingsAction::Bind(action) => {
                pending_key_bind.0 = Some(action);
            }
            SettingsAction::ResetDefaults => {
                *config = GameConfig::default();
                settings.fov_degrees = config.fov_degrees;
                settings.ui_scale = config.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
                settings.gravity_scale = config
                    .gravity_scale
                    .clamp(GRAVITY_SCALE_MIN, GRAVITY_SCALE_MAX);
                ui_scale.0 = settings.ui_scale;
                i18n.set_language(resolve_language(config.language));
                open_dropdown.0 = None;
                pending_key_bind.0 = None;
                save_config(&config);
            }
            SettingsAction::OpenFolder => {
                open_config_folder();
            }
            SettingsAction::Back => {
                open_dropdown.0 = None;
                pending_key_bind.0 = None;
                ui.unmount_panel(UiPanelId::Settings, &mut commands);
            }
        }
    }
}

fn update_settings_sliders_from_input(
    slider_changes: &Query<
        (
            &SettingsAction,
            Ref<SliderValue>,
            &SliderRange,
            &CoreSliderDragState,
        ),
        (
            With<Slider>,
            Or<(Changed<SliderValue>, Changed<CoreSliderDragState>)>,
        ),
    >,
    active_slider: &mut ActiveSettingsSlider,
    settings: &mut GameSettings,
    ui_scale: &mut UiScale,
    config: &mut GameConfig,
) {
    for (action, value, range, drag_state) in slider_changes {
        let SettingsAction::Field(field) = *action else {
            continue;
        };
        let percent = range.thumb_position(value.0).clamp(0.0, 1.0);

        if drag_state.dragging {
            active_slider.0 = Some(field);
            if field
                .slider()
                .is_some_and(|slider| slider.trigger == SettingsSliderTrigger::Live)
            {
                field.apply_percent(percent, settings, ui_scale, config);
            }
            continue;
        }

        if active_slider.0 == Some(field) || value.is_changed() {
            field.apply_percent(percent, settings, ui_scale, config);
            save_config(config);
            if active_slider.0 == Some(field) {
                active_slider.0 = None;
            }
        }
    }
}

fn commit_active_settings_slider(
    slider_values: &Query<(&SettingsAction, &SliderValue, &SliderRange), With<Slider>>,
    active_slider: &mut ActiveSettingsSlider,
    settings: &mut GameSettings,
    ui_scale: &mut UiScale,
    config: &mut GameConfig,
) {
    let Some(field) = active_slider.0.take() else {
        return;
    };

    for (action, value, range) in slider_values {
        if *action != SettingsAction::Field(field) {
            continue;
        }
        let percent = range.thumb_position(value.0).clamp(0.0, 1.0);
        field.apply_percent(percent, settings, ui_scale, config);
        save_config(config);
        return;
    }
}
