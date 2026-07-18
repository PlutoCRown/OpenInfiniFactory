use bevy::prelude::*;
use bevy::ui_widgets::{Slider, SliderDragState, SliderRange, SliderValue};
use bevy::window::PrimaryWindow;

use crate::game::state::GameSettings;
use crate::game::ui::access::UiMainThread;
use crate::game::ui::components::{
    BUTTON_BG, BUTTON_HOVER_BG, hover_border, pressed_border, raised_border, ui_logical_bounds,
};
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::types::{KeyBindingButton, UiHoverState};
use crate::shared::config::{ActionKeyName, ConfigChord, ConfigInput, GameConfig};

use super::types::{
    ActiveSettingsSlider, OpenSettingsDropdown, PendingKeyBind, SettingsAction,
    SettingsDropdownLabel, SettingsDropdownList, SettingsSliderFill, SettingsSliderKnob,
    SettingsTab, SettingsText, SettingsTextKind, SettingsValueText,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::debug_http::DebugHttpBridge;

pub fn localized_chord_display(chord: ConfigChord) -> String {
    use crate::game::ui::access::i18n;
    use crate::shared::config::primary_modifier_label;

    let mut parts = Vec::new();
    if chord.primary_modifier {
        parts.push(primary_modifier_label().to_string());
    }
    if chord.shift {
        parts.push(i18n.t("input.modifier_shift"));
    }
    if chord.alt {
        parts.push(i18n.t("input.modifier_alt"));
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
        ConfigInput::MouseLeft => i18n.t("input.mouse_left"),
        ConfigInput::MouseRight => i18n.t("input.mouse_right"),
        ConfigInput::MouseMiddle => i18n.t("input.mouse_middle"),
        ConfigInput::Key(key) => key.name().to_string(),
    }
}

pub fn update_settings_text_ui(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    config: Res<GameConfig>,
    pending_key_bind: Res<PendingKeyBind>,
    #[cfg(not(target_arch = "wasm32"))] bridge: Option<Res<DebugHttpBridge>>,
    mut primed: Local<bool>,
    mut last_bridge: Local<bool>,
    mut settings_texts: Query<(&SettingsText, Option<&ChildOf>, &mut Text)>,
    added: Query<(), Added<SettingsText>>,
    key_buttons: Query<&KeyBindingButton>,
) {
    use crate::game::ui::access::i18n;

    if !ui_runtime.is_settings_open() {
        *primed = false;
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    let bridge_running = bridge.is_some();
    #[cfg(target_arch = "wasm32")]
    let bridge_running = false;
    #[cfg(not(target_arch = "wasm32"))]
    let bridge_changed =
        bridge_running != *last_bridge || bridge.as_ref().is_some_and(|b| b.is_changed());
    #[cfg(target_arch = "wasm32")]
    let bridge_changed = false;
    *last_bridge = bridge_running;

    let dirty = !*primed
        || config.is_changed()
        || pending_key_bind.is_changed()
        || bridge_changed
        || !added.is_empty();
    if !dirty {
        return;
    }
    *primed = true;

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
                        let port = bridge.port.to_string();
                        i18n.fmt(
                            "settings.debug_http_running",
                            &[("port", port.as_str())],
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
    ui_runtime: Res<UiRuntime>,
    settings: Res<GameSettings>,
    active_slider: Res<ActiveSettingsSlider>,
    mut primed: Local<bool>,
    mut slider_fills: Query<
        (&SettingsSliderFill, &mut Node),
        (Without<SettingsSliderKnob>, Without<SettingsDropdownList>),
    >,
    mut slider_knobs: Query<
        (&SettingsSliderKnob, &mut Node),
        (Without<SettingsSliderFill>, Without<SettingsDropdownList>),
    >,
    slider_values: Query<(Entity, &SettingsAction, &SliderValue, &SliderDragState), With<Slider>>,
    added: Query<(), Added<SettingsSliderFill>>,
    mut commands: Commands,
) {
    if !ui_runtime.is_settings_open() {
        *primed = false;
        return;
    }
    // 拖动中的填充条由 update_settings_slider_drag_ui（Changed<SliderValue>）更新
    let dirty = !*primed
        || settings.is_changed()
        || active_slider.is_changed()
        || !added.is_empty();
    if !dirty {
        return;
    }
    *primed = true;

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
        let percent = fill.0.percent(&settings);
        let next = Val::Percent(percent);
        if style.width != next {
            style.width = next;
        }
    }

    for (knob, mut style) in &mut slider_knobs {
        let percent = knob.0.percent(&settings);
        let next = Val::Percent(percent);
        if style.left != next {
            style.left = next;
        }
    }
}

pub fn update_settings_slider_drag_ui(
    ui_runtime: Res<UiRuntime>,
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
    if !ui_runtime.is_settings_open() {
        return;
    }
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
    ui_runtime: Res<UiRuntime>,
    config: Res<GameConfig>,
    settings: Res<GameSettings>,
    open_dropdown: Res<OpenSettingsDropdown>,
    windows: Query<&Window, With<PrimaryWindow>>,
    changed_windows: Query<(), (With<PrimaryWindow>, Changed<Window>)>,
    mut primed: Local<bool>,
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
    added_labels: Query<(), Added<SettingsDropdownLabel>>,
    added_values: Query<(), Added<SettingsValueText>>,
) {
    if !ui_runtime.is_settings_open() {
        *primed = false;
        return;
    }

    let labels_dirty = !*primed || config.is_changed() || !added_labels.is_empty();
    let values_dirty = !*primed || settings.is_changed() || !added_values.is_empty();
    let lists_dirty =
        !*primed || open_dropdown.is_changed() || !changed_windows.is_empty();
    if !labels_dirty && !values_dirty && !lists_dirty {
        return;
    }
    *primed = true;

    if labels_dirty {
        for (label, mut text) in &mut texts.p0() {
            let next = label.0.trigger_label(&config);
            if text.0 != next {
                text.0 = next;
            }
        }
    }

    if values_dirty {
        for (value, mut text) in &mut texts.p1() {
            let next = value.0.display(&settings);
            if text.0 != next {
                text.0 = next;
            }
        }
    }

    if !lists_dirty {
        return;
    }

    let viewport = windows
        .single()
        .ok()
        .map(|window| Vec2::new(window.width(), window.height()))
        .unwrap_or(Vec2::ZERO);
    for (list, mut style, list_node) in &mut dropdown_lists {
        let open = open_dropdown.0 == Some(list.0);
        let next = if open {
            Display::Flex
        } else {
            Display::None
        };
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
    mut last_hover: Local<Option<Entity>>,
    mut tab_buttons: Query<
        (
            Entity,
            &SettingsAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    added: Query<(), Added<SettingsAction>>,
) {
    if !ui_runtime.is_settings_open() {
        *initialized = false;
        *last_hover = None;
        return;
    }

    let full_refresh = !*initialized
        || settings_tab.is_changed()
        || ui_runtime.is_changed()
        || !added.is_empty();
    if !full_refresh && hover.entity == *last_hover {
        return;
    }
    *initialized = true;

    if full_refresh {
        for (entity, action, mut background, mut border) in &mut tab_buttons {
            let selected = action.tab_selected(*settings_tab);
            let hovered = hover.entity == Some(entity);
            if selected {
                *background = Color::srgb(0.56, 0.56, 0.56).into();
                *border = pressed_border();
            } else if hovered {
                *background = BUTTON_HOVER_BG.into();
                *border = hover_border();
            } else {
                *background = BUTTON_BG.into();
                *border = raised_border();
            }
        }
        *last_hover = hover.entity;
        return;
    }

    // 仅鼠标划过变化：只重绘旧/新两个按钮
    let prev = *last_hover;
    let next = hover.entity;
    *last_hover = next;
    for entity in [prev, next].into_iter().flatten() {
        let Ok((_, action, mut background, mut border)) = tab_buttons.get_mut(entity) else {
            continue;
        };
        let selected = action.tab_selected(*settings_tab);
        let hovered = next == Some(entity);
        if selected {
            *background = Color::srgb(0.56, 0.56, 0.56).into();
            *border = pressed_border();
        } else if hovered {
            *background = BUTTON_HOVER_BG.into();
            *border = hover_border();
        } else {
            *background = BUTTON_BG.into();
            *border = raised_border();
        }
    }
}
