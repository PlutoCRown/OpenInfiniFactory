use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::{Slider, SliderDragState, SliderRange, SliderValue};
use bevy::window::PrimaryWindow;

use crate::game::state::GameSettings;
use crate::game::ui::components::{
    hover_border, pressed_border, raised_border, BUTTON_BG, BUTTON_HOVER_BG,
};
use crate::game::ui::types::{
    ActiveSettingsSlider, KeyBindingButton, LanguageChanged, OpenSettingsDropdown, PendingKeyBind,
    SettingsAction, SettingsChanged, SettingsDropdownLabel, SettingsDropdownList, SettingsField,
    SettingsSliderFill, SettingsSliderKnob, SettingsTab, SettingsText, SettingsTextKind,
    SettingsValueText, UiHoverState, UiPanelContextChanged, UiPanelKey, UiPanelOpened,
};
use crate::shared::config::GameConfig;
use crate::shared::i18n::I18n;

#[derive(SystemParam)]
pub struct SettingsPanelLifecycle<'w, 's> {
    opened: MessageReader<'w, 's, UiPanelOpened>,
    context_changed: MessageReader<'w, 's, UiPanelContextChanged>,
    settings_changed: MessageReader<'w, 's, SettingsChanged>,
    language_changed: MessageReader<'w, 's, LanguageChanged>,
}

impl SettingsPanelLifecycle<'_, '_> {
    fn dirty(&mut self) -> bool {
        self.opened
            .read()
            .any(|message| message.key == UiPanelKey::SETTINGS)
            || self
                .context_changed
                .read()
                .any(|message| message.key == UiPanelKey::SETTINGS)
            || self.settings_changed.read().next().is_some()
            || self.language_changed.read().next().is_some()
    }
}

pub fn update_settings_text_ui(
    config: Res<GameConfig>,
    pending_key_bind: Res<PendingKeyBind>,
    i18n: Res<I18n>,
    mut settings_texts: Query<(&SettingsText, Option<&ChildOf>, &mut Text)>,
    added_settings_texts: Query<(), Added<SettingsText>>,
    mut lifecycle: SettingsPanelLifecycle,
    key_buttons: Query<&KeyBindingButton>,
) {
    if !pending_key_bind.is_changed() && added_settings_texts.is_empty() && !lifecycle.dirty() {
        return;
    }

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
                format!(
                    "{}: {suffix}",
                    i18n.text(super::config_action_text_key(button.0))
                )
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
    slider_values: Query<(Entity, &SettingsAction, &SliderValue, &SliderDragState), With<Slider>>,
    added_slider_fills: Query<(), Added<SettingsSliderFill>>,
    added_slider_knobs: Query<(), Added<SettingsSliderKnob>>,
    added_sliders: Query<(), Added<Slider>>,
    mut lifecycle: SettingsPanelLifecycle,
    mut commands: Commands,
) {
    let has_active_drag = slider_values
        .iter()
        .any(|(_, _, _, drag_state)| drag_state.dragging);
    if !active_slider.is_changed()
        && !has_active_drag
        && added_slider_fills.is_empty()
        && added_slider_knobs.is_empty()
        && added_sliders.is_empty()
        && !lifecycle.dirty()
    {
        return;
    }

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
        let percent = live_slider_percent(fill.0, &settings, &active_slider, &slider_values);
        style.width = Val::Percent(percent);
    }

    for (knob, mut style) in &mut slider_knobs {
        let percent = live_slider_percent(knob.0, &settings, &active_slider, &slider_values);
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
    config: Res<GameConfig>,
    settings: Res<GameSettings>,
    open_dropdown: Res<OpenSettingsDropdown>,
    i18n: Res<I18n>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    added_dropdown_labels: Query<(), Added<SettingsDropdownLabel>>,
    added_value_texts: Query<(), Added<SettingsValueText>>,
    added_dropdown_lists: Query<(), Added<SettingsDropdownList>>,
    mut lifecycle: SettingsPanelLifecycle,
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
    let dropdown_open = open_dropdown.0.is_some();
    let lifecycle_dirty = lifecycle.dirty();
    let refresh_dropdown_labels = !added_dropdown_labels.is_empty() || lifecycle_dirty;
    let refresh_value_texts = !added_value_texts.is_empty() || lifecycle_dirty;
    let refresh_dropdown_layout = dropdown_open
        || open_dropdown.is_changed()
        || ui_scale.is_changed()
        || !added_dropdown_lists.is_empty()
        || lifecycle_dirty;

    if !refresh_dropdown_labels && !refresh_value_texts && !refresh_dropdown_layout {
        return;
    }

    if refresh_dropdown_labels {
        for (label, mut text) in &mut texts.p0() {
            let Some(spec) = super::settings_dropdown_spec_by_id(label.0) else {
                continue;
            };
            text.0 = super::settings_dropdown_value_text(spec, &config, &i18n);
        }
    }

    if refresh_value_texts {
        for (value, mut text) in &mut texts.p1() {
            text.0 = value.0.display(&settings, &i18n);
        }
    }

    if !refresh_dropdown_layout {
        return;
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|window| Vec2::new(window.width(), window.height()))
        .unwrap_or(Vec2::ZERO);
    let scale = settings_ui_transform_scale(window, ui_scale.0);
    for (list, mut style, list_node) in &mut dropdown_lists {
        let open = open_dropdown.0 == Some(list.0);
        style.display = if open { Display::Flex } else { Display::None };
        if !open {
            continue;
        }
        if let Some((left, top)) = dropdown_position(
            SettingsAction::ToggleDropdown(list.0),
            &triggers,
            list_node.size(),
            viewport,
            scale,
        ) {
            style.left = Val::Px(left);
            style.top = Val::Px(top);
        }
    }
}

fn dropdown_position(
    target: SettingsAction,
    triggers: &Query<(&SettingsAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
    list_size: Vec2,
    viewport: Vec2,
    scale: f32,
) -> Option<(f32, f32)> {
    let (_, trigger_node, transform) = triggers
        .iter()
        .find(|(action, node, _)| **action == target && !node.is_empty())?;
    let trigger_size = trigger_node.size();
    let center = (*transform * Vec2::ZERO) * scale;
    let trigger_left = center.x - trigger_size.x * 0.5;
    let trigger_top = center.y - trigger_size.y * 0.5;
    let below = trigger_top + trigger_size.y + 4.0;
    let above = trigger_top - list_size.y - 4.0;
    let top = if below + list_size.y <= viewport.y - 10.0 || above < 10.0 {
        below
    } else {
        above.max(10.0)
    };
    let left = trigger_left.clamp(10.0, (viewport.x - list_size.x - 10.0).max(10.0));
    Some((left, top))
}

fn settings_ui_transform_scale(window: Option<&Window>, ui_scale: f32) -> f32 {
    window.map(Window::scale_factor).unwrap_or(1.0) / ui_scale.max(0.01)
}

pub fn update_settings_tabs_ui(
    settings_tab: Res<SettingsTab>,
    hover: Res<UiHoverState>,
    added_tabs: Query<(), (Added<SettingsAction>, With<Button>)>,
    mut lifecycle: SettingsPanelLifecycle,
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
    if !settings_tab.is_changed()
        && !hover.is_changed()
        && added_tabs.is_empty()
        && !lifecycle.dirty()
    {
        return;
    }

    for (entity, action, mut background, mut border) in &mut tab_buttons {
        let selected = matches!(
            (*action, *settings_tab),
            (SettingsAction::TabGameplay, SettingsTab::Gameplay)
                | (SettingsAction::TabKeyBindings, SettingsTab::KeyBindings)
        );
        let hovered = hover.entity == Some(entity);
        if selected {
            *background = Color::srgb(0.56, 0.56, 0.56).into();
            *border = pressed_border();
        } else if matches!(
            *action,
            SettingsAction::TabGameplay | SettingsAction::TabKeyBindings
        ) {
            if hovered {
                *background = BUTTON_HOVER_BG.into();
                *border = hover_border();
            } else {
                *background = BUTTON_BG.into();
                *border = raised_border();
            }
        } else {
            if hovered {
                *background = BUTTON_HOVER_BG.into();
                *border = hover_border();
            } else {
                *background = BUTTON_BG.into();
                *border = raised_border();
            }
        }
    }
}

fn live_slider_percent(
    field: SettingsField,
    settings: &GameSettings,
    active_slider: &ActiveSettingsSlider,
    slider_values: &Query<(Entity, &SettingsAction, &SliderValue, &SliderDragState), With<Slider>>,
) -> f32 {
    slider_values
        .iter()
        .find_map(|(_, action, value, drag_state)| {
            ((drag_state.dragging || active_slider.0 == Some(field))
                && *action == SettingsAction::Field(field))
            .then_some(value.0.clamp(0.0, 100.0))
        })
        .unwrap_or_else(|| field.percent(settings))
}
