use bevy::prelude::*;
use bevy::ui_widgets::{CoreSliderDragState, Slider, SliderRange, SliderValue};
use bevy::window::PrimaryWindow;

use crate::game::state::GameSettings;
use crate::game::ui::components::{
    hover_border, pressed_border, raised_border, ui_logical_bounds, BUTTON_BG, BUTTON_HOVER_BG,
};
use crate::game::ui::types::{KeyBindingButton, UiHoverState};
use crate::shared::config::GameConfig;

use crate::game::ui::access::UiMainThread;
use super::types::{
    ActiveSettingsSlider, OpenSettingsDropdown, PendingKeyBind, SettingsAction,
    SettingsDropdownLabel, SettingsDropdownList, SettingsField, SettingsSliderFill,
    SettingsSliderKnob, SettingsTab, SettingsText, SettingsTextKind, SettingsValueText,
};

pub fn update_settings_text_ui(
    _ui_thread: UiMainThread,
    config: Res<GameConfig>,
    pending_key_bind: Res<PendingKeyBind>,
    mut settings_texts: Query<(&SettingsText, Option<&ChildOf>, &mut Text)>,
    key_buttons: Query<&KeyBindingButton>,
) {
    use crate::game::ui::access::i18n;

    for (settings_text, parent, mut text) in &mut settings_texts {
        text.0 = match settings_text.0 {
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
                    .map(|_| "...")
                    .unwrap_or(config.input(button.0).name());
                format!("{}: {suffix}", i18n.t(button.0.label_key()))
            }
        };
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
    slider_values: Query<
        (Entity, &SettingsAction, &SliderValue, &CoreSliderDragState),
        With<Slider>,
    >,
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
        style.width = Val::Percent(percent);
    }

    for (knob, mut style) in &mut slider_knobs {
        let percent = live_slider_percent(knob.0, &settings, &slider_values);
        style.left = Val::Percent(percent);
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
        text.0 = label.0.trigger_label(&config);
    }

    for (value, mut text) in &mut texts.p1() {
        text.0 = value.0.display(&settings);
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|window| Vec2::new(window.width(), window.height()))
        .unwrap_or(Vec2::ZERO);
    for (list, mut style, list_node) in &mut dropdown_lists {
        let open = open_dropdown.0 == Some(list.0);
        style.display = if open { Display::Flex } else { Display::None };
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
    settings_tab: Res<SettingsTab>,
    hover: Res<UiHoverState>,
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
    slider_values: &Query<
        (Entity, &SettingsAction, &SliderValue, &CoreSliderDragState),
        With<Slider>,
    >,
) -> f32 {
    slider_values
        .iter()
        .find_map(|(_, action, value, drag_state)| {
            (drag_state.dragging && *action == SettingsAction::Field(field))
                .then_some(value.0.clamp(0.0, 100.0))
        })
        .unwrap_or_else(|| field.percent(settings))
}
