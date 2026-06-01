use bevy::prelude::*;

use crate::game::ui::components::{
    default_button_size, full_width_button, label_text, raised_border, slider_bundle, slider_fill,
    slider_knob, styled_button, transparent_node,
};
use crate::game::ui::types::{
    KeyBindingButton, LocalizedText, SettingsAction, SettingsDropdownId, SettingsDropdownLabel,
    SettingsDropdownList, SettingsField, SettingsSliderFill, SettingsSliderKnob, SettingsText,
    SettingsTextKind, SettingsValueText,
};

pub(super) fn spawn_localized_settings_button(
    parent: &mut ChildSpawnerCommands,
    action: SettingsAction,
    text_key: &'static str,
) {
    let is_binding = matches!(action, SettingsAction::Bind(_));
    let mut button = parent.spawn((full_width_button(36.0), action));
    if let SettingsAction::Bind(action) = action {
        button.insert(KeyBindingButton(action));
    }
    button.with_children(|button| {
        let mut label_entity = button.spawn((
            label_text(text_key, 14.0, Color::WHITE),
            LocalizedText { key: text_key },
        ));
        if is_binding {
            label_entity.insert(SettingsText(SettingsTextKind::KeyBinding));
        }
    });
}

pub(super) fn spawn_settings_tab(
    parent: &mut ChildSpawnerCommands,
    action: SettingsAction,
    text_key: &'static str,
) {
    parent
        .spawn((
            styled_button(
                Node {
                    min_width: Val::Px(default_button_size(150.0)),
                    height: Val::Px(default_button_size(38.0)),
                    padding: UiRect::horizontal(Val::Px(18.0)),
                    border: UiRect {
                        left: Val::Px(1.0),
                        right: Val::Px(1.0),
                        top: Val::Px(1.0),
                        bottom: Val::Px(3.0),
                    },
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                raised_border(),
                Color::srgb(0.255, 0.251, 0.251),
            ),
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.45),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(3.0),
            ),
            action,
        ))
        .with_children(|tab| {
            tab.spawn((
                label_text(text_key, 15.0, Color::WHITE),
                LocalizedText { key: text_key },
            ));
        });
}

pub(super) fn spawn_settings_slider(parent: &mut ChildSpawnerCommands, field: SettingsField) {
    parent
        .spawn(slider_bundle(SettingsAction::Field(field)))
        .with_children(|track| {
            track.spawn((slider_fill(), SettingsSliderFill(field)));
            track.spawn((slider_knob(), SettingsSliderKnob(field)));
        });
}

pub(super) fn spawn_settings_slider_value(parent: &mut ChildSpawnerCommands, field: SettingsField) {
    parent.spawn((
        label_text("", 13.0, Color::srgb(0.88, 0.94, 0.96)),
        TextLayout::new_with_justify(Justify::Right),
        Node {
            width: Val::Px(130.0),
            align_self: AlignSelf::Center,
            ..default()
        },
        SettingsValueText(field),
    ));
}

pub(super) fn spawn_settings_dropdown(
    parent: &mut ChildSpawnerCommands,
    dropdown: SettingsDropdownId,
) {
    parent
        .spawn((
            transparent_node(Node {
                width: Val::Px(260.0),
                position_type: PositionType::Relative,
                ..default()
            }),
            ZIndex(300),
        ))
        .with_children(|container| {
            container
                .spawn((
                    styled_button(
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(default_button_size(36.0)),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        Color::srgb(0.38, 0.39, 0.40),
                        Color::srgba(0.18, 0.20, 0.22, 0.96),
                    ),
                    SettingsAction::ToggleDropdown(dropdown),
                ))
                .with_children(|button| {
                    button.spawn((
                        label_text("", 14.0, Color::WHITE),
                        SettingsDropdownLabel(dropdown),
                    ));
                    button.spawn(label_text("v", 12.0, Color::srgb(0.72, 0.80, 0.84)));
                });
        });
}

pub(super) fn spawn_settings_dropdown_list(
    parent: &mut ChildSpawnerCommands,
    dropdown: SettingsDropdownId,
    options: impl IntoIterator<Item = (String, SettingsAction)>,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(260.0),
                display: Display::None,
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98)),
            GlobalZIndex(20_000),
            SettingsDropdownList(dropdown),
        ))
        .with_children(|list| {
            for (label, action) in options {
                list.spawn((crate::game::ui::components::menu_button(32.0), action))
                    .with_children(|button| {
                        button.spawn(label_text(label, 13.0, Color::WHITE));
                    });
            }
        });
}
