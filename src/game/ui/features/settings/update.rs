use bevy::prelude::*;
use bevy::ui_widgets::{Slider, SliderDragState, SliderRange, SliderValue};
use bevy::window::PrimaryWindow;

use crate::game::state::GameSettings;
use crate::game::ui::components::{
    BUTTON_BG, BUTTON_HOVER_BG, hover_border, pressed_border, raised_border, ui_logical_bounds,
};
use crate::game::ui::types::{KeyBindingButton, UiHoverState};

use super::types::{
    ActiveSettingsSlider, OpenSettingsDropdown, PendingKeyBind, SettingsAction,
    SettingsDropdownLabel, SettingsDropdownList, SettingsField, SettingsSliderFill,
    SettingsSliderKnob, SettingsTab, SettingsText, SettingsTextKind, SettingsValueText,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::debug_http::DebugHttpBridge;
use crate::game::ui::access::UiMainThread;
use crate::game::ui::core::runtime::UiRuntime;
use crate::shared::config::{ActionKeyName, ConfigChord, ConfigInput, GameConfig};

pub fn localized_chord_display(chord: ConfigChord) -> String {
    use crate::game::ui::access::i18n;
    use crate::shared::config::primary_modifier_label;

    let mut parts = Vec::new();
    if chord.primary_modifier {
        parts.push(primary_modifier_label().to_string());
    }
    if chord.shift {
        parts.push(i18n.t("input.modifier_shift").to_string());
    }
    if chord.alt {
        parts.push(i18n.t("input.modifier_alt").to_string());
    }
    parts.push(chord.key.name().to_string());
    parts.join("+")
}

pub fn localized_binding_display(config: &GameConfig, action: ActionKeyName) -> String {
    use crate::game::ui::access::i18n;

    if action.is_chord() {
        return localized_chord_display(config.chord(action));
    }
    match config.input(action) {
        ConfigInput::MouseLeft => i18n.t("input.mouse_left").to_string(),
        ConfigInput::MouseRight => i18n.t("input.mouse_right").to_string(),
        ConfigInput::MouseMiddle => i18n.t("input.mouse_middle").to_string(),
        ConfigInput::Key(key) => key.name().to_string(),
    }
}

pub fn update_settings_text_ui(
    _ui_thread: UiMainThread,
    config: Res<GameConfig>,
    pending_key_bind: Res<PendingKeyBind>,
    #[cfg(not(target_arch = "wasm32"))] bridge: Option<Res<DebugHttpBridge>>,
    mut settings_texts: Query<(&SettingsText, Option<&ChildOf>, &mut Text)>,
    key_buttons: Query<&KeyBindingButton>,
) {
    use crate::game::ui::access::i18n;

    for (settings_text, parent, mut text) in &mut settings_texts {
        let next = match settings_text.0 {
            SettingsTextKind::KeyBinding => {
                let Some(parent) = parent else {
                    continue;
                };
                let Ok(button) = key_buttons.get(parent.parent()) else {
                    continue;
                };
                let suffix = pending_key_bind
                    .0
                    .filter(|pending| *pending == button.0)
                    .map(|_| "...".to_string())
                    .unwrap_or_else(|| localized_binding_display(&config, button.0));
                format!("{}: {suffix}", i18n.t(button.0.label_key()))
            }
            SettingsTextKind::DebugHttp => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(bridge) = bridge.as_ref() {
                        i18n.fmt(
                            "settings.debug_http_running",
                            &[("port", bridge.port.to_string())],
                        )
                    } else {
                        i18n.t("button.start_debug_http")
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    String::new()
                }
            }
        };
        if text.0 != next {
            text.0 = next;
        }
    }
}

pub fn update_settings_sliders_ui(
    settings: Res<GameSettings>,
    active_slider: Res<ActiveSettingsSlider>,
    mut slider_fills: Query<
        (&SettingsSliderFill, &mut Node),
        (Without<SettingsSliderKnob>, Without<SettingsDropdownList>),
    >,
    mut slider_knobs: Query<
        (&SettingsSliderKnob, &mut Node),
        (Without<SettingsSliderFill>, Without<SettingsDropdownList>),
    >,
    slider_values: Query<(Entity, &SettingsAction, &SliderValue, &SliderDragState), With<Slider>>,
    mut commands: Commands,
) {
    for (entity, action, value, drag_state) in &slider_values {
        if drag_state.dragging {
            continue;
        }
        if let SettingsAction::Field(field) = *action {
            if active_slider.0 == Some(field) {
                continue;
            }
            let next_value = field.percent(&settings);
            if (value.0 - next_value).abs() > 0.01 {
                commands.entity(entity).insert(SliderValue(next_value));
            }
        }
    }

    for (fill, mut style) in &mut slider_fills {
        let percent = live_slider_percent(fill.0, &settings, &slider_values);
        let next = Val::Percent(percent);
        if style.width != next {
            style.width = next;
        }
    }

    for (knob, mut style) in &mut slider_knobs {
        let percent = live_slider_percent(knob.0, &settings, &slider_values);
        let next = Val::Percent(percent);
        if style.left != next {
            style.left = next;
        }
    }
}

pub fn update_settings_slider_drag_ui(
    slider_values: Query<
        (&SettingsAction, &SliderValue, &SliderRange),
        (With<Slider>, Changed<SliderValue>),
    >,
    mut slider_fills: Query<
        (&SettingsSliderFill, &mut Node),
        (Without<SettingsSliderKnob>, Without<SettingsDropdownList>),
    >,
    mut slider_knobs: Query<
        (&SettingsSliderKnob, &mut Node),
        (Without<SettingsSliderFill>, Without<SettingsDropdownList>),
    >,
) {
    for (action, value, range) in &slider_values {
        let SettingsAction::Field(field) = *action else {
            continue;
        };
        let percent = (range.thumb_position(value.0) * 100.0).clamp(0.0, 100.0);

        for (fill, mut style) in &mut slider_fills {
            if fill.0 == field {
                style.width = Val::Percent(percent);
            }
        }

        for (knob, mut style) in &mut slider_knobs {
            if knob.0 == field {
                style.left = Val::Percent(percent);
            }
        }
    }
}

pub fn update_settings_dropdowns_ui(
    _ui_thread: UiMainThread,
    config: Res<GameConfig>,
    settings: Res<GameSettings>,
    open_dropdown: Res<OpenSettingsDropdown>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut texts: ParamSet<(
        Query<
            (&SettingsDropdownLabel, &mut Text),
            (Without<SettingsText>, Without<SettingsValueText>),
        >,
        Query<
            (&SettingsValueText, &mut Text),
            (Without<SettingsText>, Without<SettingsDropdownLabel>),
        >,
    )>,
    mut dropdown_lists: Query<
        (&SettingsDropdownList, &mut Node, &ComputedNode),
        (Without<SettingsSliderFill>, Without<SettingsSliderKnob>),
    >,
    triggers: Query<(&SettingsAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    for (label, mut text) in &mut texts.p0() {
        let next = label.0.trigger_label(&config);
        if text.0 != next {
            text.0 = next;
        }
    }

    for (value, mut text) in &mut texts.p1() {
        let next = value.0.display(&settings);
        if text.0 != next {
            text.0 = next;
        }
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|window| Vec2::new(window.width(), window.height()))
        .unwrap_or(Vec2::ZERO);
    for (list, mut style, list_node) in &mut dropdown_lists {
        let open = open_dropdown.0 == Some(list.0);
        let next = if open { Display::Flex } else { Display::None };
        if style.display != next {
            style.display = next;
        }
        if !open {
            continue;
        }
        if let Some((left, top)) = dropdown_position(
            SettingsAction::ToggleDropdown(list.0),
            &triggers,
            list_node,
            viewport,
        ) {
            style.left = Val::Px(left);
            style.top = Val::Px(top);
        }
    }
}

fn dropdown_position(
    target: SettingsAction,
    triggers: &Query<(&SettingsAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
    list_node: &ComputedNode,
    viewport: Vec2,
) -> Option<(f32, f32)> {
    let (_, trigger_node, transform) = triggers
        .iter()
        .find(|(action, node, _)| **action == target && !node.is_empty())?;
    let trigger = ui_logical_bounds(trigger_node, transform);
    let list_size = list_node.size() * list_node.inverse_scale_factor();
    let below = trigger.max.y + 4.0;
    let above = trigger.min.y - list_size.y - 4.0;
    let top = if below + list_size.y <= viewport.y - 10.0 || above < 10.0 {
        below
    } else {
        above.max(10.0)
    };
    let top = top.clamp(10.0, (viewport.y - list_size.y - 10.0).max(10.0));
    let left = trigger
        .min
        .x
        .clamp(10.0, (viewport.x - list_size.x - 10.0).max(10.0));
    Some((left, top))
}

pub fn update_settings_tabs_ui(
    ui_runtime: Res<UiRuntime>,
    settings_tab: Res<SettingsTab>,
    hover: Res<UiHoverState>,
    mut initialized: Local<bool>,
    mut tab_buttons: Query<
        (
            Entity,
            &SettingsAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    if !ui_runtime.is_settings_open() {
        *initialized = false;
        return;
    }
    if *initialized && !settings_tab.is_changed() && !hover.is_changed() && !ui_runtime.is_changed()
    {
        return;
    }
    *initialized = true;

    for (entity, action, mut background, mut border) in &mut tab_buttons {
        let selected = action.tab_selected(*settings_tab);
        let hovered = hover.entity == Some(entity);
        if selected {
            *background = Color::srgb(0.56, 0.56, 0.56).into();
            *border = pressed_border();
        } else if action.is_tab() {
            if hovered {
                *background = BUTTON_HOVER_BG.into();
                *border = hover_border();
            } else {
                *background = BUTTON_BG.into();
                *border = raised_border();
            }
        } else if hovered {
            *background = BUTTON_HOVER_BG.into();
            *border = hover_border();
        } else {
            *background = BUTTON_BG.into();
            *border = raised_border();
        }
    }
}

fn live_slider_percent(
    field: SettingsField,
    settings: &GameSettings,
    slider_values: &Query<(Entity, &SettingsAction, &SliderValue, &SliderDragState), With<Slider>>,
) -> f32 {
    slider_values
        .iter()
        .find_map(|(_, action, value, drag_state)| {
            (drag_state.dragging && *action == SettingsAction::Field(field))
                .then_some(value.0.clamp(0.0, 100.0))
        })
        .unwrap_or_else(|| field.percent(settings))
}
