use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::ui_widgets::{Slider, SliderDragState, SliderRange, SliderValue};

use crate::game::state::GameSettings;
use crate::game::systems::world_flow::primary_click;
use crate::game::ui::{
    ActiveSettingsSlider, CloseUiPanel, LanguageChanged, OpenSettingsDropdown, PendingKeyBind,
    SettingsAction, SettingsChanged, SettingsSliderTrigger, SettingsTab, UiPanelClosed,
    UiPanelKey, UiRuntime,
};
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{input_from_buttons, open_config_folder, save_config, GameConfig};
use crate::shared::i18n::{resolve_language, I18n};

pub fn settings_menu_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    ui_runtime: Res<UiRuntime>,
    mut settings_changed: MessageWriter<SettingsChanged>,
    slider_values: Query<(&SettingsAction, &SliderValue, &SliderRange), With<Slider>>,
    slider_changes: Query<
        (
            &SettingsAction,
            Ref<SliderValue>,
            &SliderRange,
            &SliderDragState,
        ),
        (
            With<Slider>,
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
        ),
    >,
) {
    if !ui_runtime.is_settings_open() {
        active_slider.0 = None;
        return;
    }

    if let Some(action) = pending_key_bind.0 {
        if let Some(input) = input_from_buttons(&keys, &mouse_buttons) {
            config.set_input(action, input);
            save_config(&config);
            settings_changed.write(SettingsChanged);
            pending_key_bind.0 = None;
        }
    }

    update_settings_sliders_from_input(
        &slider_changes,
        &mut active_slider,
        &mut settings,
        &mut ui_scale,
        &mut config,
        &mut settings_changed,
    );

    if mouse_buttons.just_released(MouseButton::Left) {
        commit_active_settings_slider(
            &slider_values,
            &mut active_slider,
            &mut settings,
            &mut ui_scale,
            &mut config,
            &mut settings_changed,
        );
    }
}

pub fn cleanup_closed_settings_panel(
    mut closed: MessageReader<UiPanelClosed>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
) {
    if !closed
        .read()
        .any(|message| message.key == UiPanelKey::SETTINGS)
    {
        return;
    }
    open_dropdown.0 = None;
    pending_key_bind.0 = None;
    active_slider.0 = None;
}

pub fn settings_action_clicked(
    mut click: On<Pointer<Click>>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    mut i18n: ResMut<I18n>,
    mut settings_tab: ResMut<SettingsTab>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    ui_runtime: Res<UiRuntime>,
    mut close_panel: MessageWriter<CloseUiPanel>,
    mut settings_changed: MessageWriter<SettingsChanged>,
    mut language_changed: MessageWriter<LanguageChanged>,
    actions: Query<&SettingsAction>,
) {
    if !primary_click(&mut click) || !ui_runtime.is_settings_open() {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

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
            settings_changed.write(SettingsChanged);
        }
        SettingsAction::SetDeleteSelectionMode(selection_mode) => {
            config.delete_selection_mode = selection_mode;
            open_dropdown.0 = None;
            save_config(&config);
            settings_changed.write(SettingsChanged);
        }
        SettingsAction::SetLanguage(language) => {
            i18n.set_language(language);
            config.language = Some(language);
            open_dropdown.0 = None;
            save_config(&config);
            settings_changed.write(SettingsChanged);
            language_changed.write(LanguageChanged);
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
            settings_changed.write(SettingsChanged);
            language_changed.write(LanguageChanged);
        }
        SettingsAction::OpenFolder => {
            open_config_folder();
        }
        SettingsAction::Back => {
            open_dropdown.0 = None;
            pending_key_bind.0 = None;
            close_panel.write(CloseUiPanel { key: None });
        }
    }
}

fn update_settings_sliders_from_input(
    slider_changes: &Query<
        (
            &SettingsAction,
            Ref<SliderValue>,
            &SliderRange,
            &SliderDragState,
        ),
        (
            With<Slider>,
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
        ),
    >,
    active_slider: &mut ActiveSettingsSlider,
    settings: &mut GameSettings,
    ui_scale: &mut UiScale,
    config: &mut GameConfig,
    settings_changed: &mut MessageWriter<SettingsChanged>,
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
                settings_changed.write(SettingsChanged);
            }
            continue;
        }

        if active_slider.0 == Some(field) || value.is_changed() {
            field.apply_percent(percent, settings, ui_scale, config);
            save_config(config);
            settings_changed.write(SettingsChanged);
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
    settings_changed: &mut MessageWriter<SettingsChanged>,
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
        settings_changed.write(SettingsChanged);
        return;
    }
}
