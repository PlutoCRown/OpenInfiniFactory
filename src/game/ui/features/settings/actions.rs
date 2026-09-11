use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Pointer, Release};
use bevy::prelude::*;
use bevy::ui_widgets::{Slider, SliderRange, ValueChange};

#[cfg(not(target_arch = "wasm32"))]
use crate::debug_http::PendingDebugHttpStart;
use crate::game::state::GameSettings;
use crate::game::ui::access::{UiContext, ui};
use crate::game::ui::core::host::{UiAction, UiInstanceId};
use crate::game::ui::core::runtime::UiNavigation;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::settings::confirm::{on_reset_defaults, reset_defaults_spec};
use crate::shared::config::{
    GameConfig, chord_from_input, input_from_buttons, open_config_folder, save_config,
};
use crate::shared::touch_profile::TouchProfile;

use super::types::{
    ActiveSettingsSlider, OpenSettingsDropdown, PendingKeyBind, SettingsAction,
    SettingsSliderTrigger, SettingsTab,
};

/// 设置页脚按钮绑定局部操作，不要求分发系统预借用业务资源。
struct SettingsFooterButton {
    action: SettingsAction,
    on_click: fn(&mut Commands),
}

const SETTINGS_FOOTER: &[SettingsFooterButton] = &[
    SettingsFooterButton {
        action: SettingsAction::ResetDefaults,
        on_click: |commands| {
            commands.run_system_cached(confirm_reset_defaults);
        },
    },
    SettingsFooterButton {
        action: SettingsAction::OpenFolder,
        on_click: |_commands| open_config_folder(),
    },
    SettingsFooterButton {
        action: SettingsAction::StartDebugHttp,
        on_click: |commands| {
            #[cfg(not(target_arch = "wasm32"))]
            commands.insert_resource(PendingDebugHttpStart(true));
        },
    },
];

/// 重置设置按钮系统只在需要确认时获取本地化上下文。
fn confirm_reset_defaults(ui_context: UiContext, mut commands: Commands) {
    let _ui_scope = ui_context.enter();
    ui.open_confirm_then(&mut commands, reset_defaults_spec(), on_reset_defaults);
}

pub fn settings_menu_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut config: ResMut<GameConfig>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    ui_navigation: Res<UiNavigation>,
) {
    if !ui_navigation.is_settings_open() {
        pending_key_bind.0 = None;
        open_dropdown.0 = None;
        active_slider.clear();
        return;
    }

    if let Some(action) = pending_key_bind.0 {
        if action.is_chord() {
            if let Some(chord) = chord_from_input(&keys) {
                config.set_chord(action, chord);
                save_config(&config);
                pending_key_bind.0 = None;
            }
        } else if let Some(input) = input_from_buttons(&keys, &mouse_buttons) {
            config.set_input(action, input);
            save_config(&config);
            pending_key_bind.0 = None;
        }
    }
}

/// 接收滑条的预览/最终值：Live 实时应用，Commit 只在最终确认时应用
pub fn settings_slider_changed(
    change: On<ValueChange<f32>>,
    sliders: Query<(&SettingsAction, &SliderRange), With<Slider>>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    touch: Res<TouchProfile>,
) {
    let Ok((SettingsAction::Field(field), range)) = sliders.get(change.source) else {
        return;
    };
    let field = *field;
    let percent = range.thumb_position(change.value).clamp(0.0, 1.0);
    active_slider.field = Some(field);
    active_slider.percent = percent;

    let live = field
        .slider()
        .is_some_and(|slider| slider.trigger == SettingsSliderTrigger::Live);
    if live || change.is_final {
        field.apply_percent(percent, &mut settings, &mut ui_scale, &mut config, *touch);
    }
    if change.is_final {
        save_config(&config);
        active_slider.clear();
    }
}

/// 轨道单击没有最终 ValueChange，指针松开时提交当前预览值
pub fn settings_slider_released(
    release: On<Pointer<Release>>,
    sliders: Query<(), With<Slider>>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    touch: Res<TouchProfile>,
) {
    if release.event.button != PointerButton::Primary || !sliders.contains(release.entity) {
        return;
    }
    let Some(field) = active_slider.field else {
        return;
    };
    field.apply_percent(
        active_slider.percent,
        &mut settings,
        &mut ui_scale,
        &mut config,
        *touch,
    );
    save_config(&config);
    active_slider.clear();
}

pub fn emit_settings_actions(
    mut click: On<Pointer<Click>>,
    ui_navigation: Res<UiNavigation>,
    mut writer: MessageWriter<UiAction<SettingsAction>>,
    actions: Query<&SettingsAction>,
) {
    if ui_navigation.modal().is_some()
        || !primary_click(&mut click)
        || !ui_navigation.is_settings_open()
    {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    writer.write(UiAction {
        instance: UiInstanceId::SETTINGS,
        kind: action,
    });
}

pub fn dispatch_settings_actions(
    mut actions: MessageReader<UiAction<SettingsAction>>,
    mut config: ResMut<GameConfig>,
    mut settings_tab: ResMut<SettingsTab>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut commands: Commands,
) {
    for action in actions.read() {
        if action.instance != UiInstanceId::SETTINGS {
            continue;
        }
        let action = action.kind.clone();
        if let Some(entry) = SETTINGS_FOOTER.iter().find(|entry| entry.action == action) {
            (entry.on_click)(&mut commands);
            continue;
        }
        match action {
            SettingsAction::TabGameplay => {
                *settings_tab = SettingsTab::Gameplay;
                open_dropdown.0 = None;
            }
            SettingsAction::TabGraphics => {
                *settings_tab = SettingsTab::Graphics;
                open_dropdown.0 = None;
            }
            SettingsAction::TabKeyBindings => {
                *settings_tab = SettingsTab::KeyBindings;
                open_dropdown.0 = None;
            }
            SettingsAction::TabAudio => {
                *settings_tab = SettingsTab::Audio;
                open_dropdown.0 = None;
            }
            // 滑条通过 ValueChange 维护预览与提交，Click 不是数值状态来源
            SettingsAction::Field(_) => {}
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
                config.language = Some(language);
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetShadowsEnabled(enabled) => {
                config.shadows_enabled = enabled;
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetSsaoQuality(quality) => {
                config.ssao_quality = quality;
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetGameplayRenderRate(rate) => {
                config.gameplay_render_rate = rate;
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetVsyncEnabled(enabled) => {
                config.vsync_enabled = enabled;
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetSkyboxEnabled(enabled) => {
                config.skybox_enabled = enabled;
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetWindowMode(mode) => {
                config.window_mode = mode;
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
            SettingsAction::OpenVirtualLayout => {
                commands.queue(|world: &mut World| {
                    crate::game::ui::features::virtual_remote::open_virtual_layout_editor(world);
                });
            }
            SettingsAction::ResetDefaults
            | SettingsAction::OpenFolder
            | SettingsAction::StartDebugHttp => {}
        }
    }
}
