use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::ui::components::{
    default_button_size, default_font_size, raised_border, slider_bundle, slider_fill, slider_knob,
    HoverButton, BUTTON_BG,
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
    let mut button = parent.spawn((Button, HoverButton, action));
    if let SettingsAction::Bind(action) = action {
        button.insert(KeyBindingButton(action));
    }
    button
        .queue_apply_scene(settings_full_width_button_scene(36.0))
        .with_children(|button| {
            let mut label_entity = button.spawn(LocalizedText { key: text_key });
            label_entity.queue_apply_scene(settings_button_label_scene(text_key, 14.0));
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
        .spawn((Button, HoverButton, action))
        .queue_apply_scene(settings_tab_scene())
        .queue_spawn_related_scenes::<Children>(settings_localized_label_scene(text_key, 15.0));
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
    parent
        .spawn(SettingsValueText(field))
        .queue_apply_scene(settings_slider_value_scene());
}

pub(super) fn spawn_settings_dropdown(
    parent: &mut ChildSpawnerCommands,
    dropdown: SettingsDropdownId,
) {
    parent
        .spawn_empty()
        .queue_apply_scene(settings_dropdown_container_scene())
        .with_children(|container| {
            container
                .spawn((
                    Button,
                    HoverButton,
                    SettingsAction::ToggleDropdown(dropdown),
                ))
                .queue_apply_scene(settings_dropdown_button_scene())
                .with_children(|button| {
                    button
                        .spawn(SettingsDropdownLabel(dropdown))
                        .queue_apply_scene(settings_dropdown_label_scene());
                    button
                        .spawn_empty()
                        .queue_apply_scene(settings_dropdown_caret_scene());
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
                list.spawn((Button, HoverButton, action))
                    .queue_apply_scene(settings_menu_button_scene(32.0))
                    .queue_spawn_related_scenes::<Children>(settings_owned_label_scene(
                        label, 13.0,
                    ));
            }
        });
}

fn settings_full_width_button_scene(height: f32) -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            height: Val::Px(default_button_size(height)),
            border: UiRect {
                left: Val::Px(3.0),
                right: Val::Px(3.0),
                top: Val::Px(4.0),
                bottom: Val::Px(5.0),
            },
            padding: UiRect::horizontal(Val::Px(14.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(BUTTON_BG)
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.62),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(4.0),
        )
    }
}

fn settings_menu_button_scene(height: f32) -> impl bevy_scene::Scene {
    bsn! {
        Node {
            height: Val::Px(default_button_size(height)),
            border: UiRect {
                left: Val::Px(3.0),
                right: Val::Px(3.0),
                top: Val::Px(4.0),
                bottom: Val::Px(5.0),
            },
            padding: UiRect::horizontal(Val::Px(14.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(BUTTON_BG)
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.62),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(4.0),
        )
    }
}

fn settings_button_label_scene(text_key: &'static str, font_size: f32) -> impl bevy_scene::Scene {
    bsn! {
        Text({text_key})
        TextFont {
            font_size: {default_font_size(font_size)}
        }
        TextColor(Color::WHITE)
    }
}

fn settings_localized_label_scene(
    text_key: &'static str,
    font_size: f32,
) -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text({text_key})
            TextFont {
                font_size: {default_font_size(font_size)}
            }
            TextColor(Color::WHITE)
            LocalizedText {
                key: {text_key}
            }
        )
    }
}

fn settings_owned_label_scene(label: String, font_size: f32) -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text({label})
            TextFont {
                font_size: {default_font_size(font_size)}
            }
            TextColor(Color::WHITE)
        )
    }
}

fn settings_tab_scene() -> impl bevy_scene::Scene {
    bsn! {
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
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(Color::srgb(0.255, 0.251, 0.251))
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.45),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(3.0),
        )
    }
}

fn settings_slider_value_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        TextFont {
            font_size: {default_font_size(13.0)}
        }
        TextColor(Color::srgb(0.88, 0.94, 0.96))
        TextLayout::justify(Justify::Right)
        Node {
            width: Val::Px(130.0),
            align_self: AlignSelf::Center,
        }
    }
}

fn settings_dropdown_container_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Px(260.0),
            position_type: PositionType::Relative,
        }
        ZIndex(300)
        BackgroundColor(Color::NONE)
    }
}

fn settings_dropdown_button_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(default_button_size(36.0)),
            padding: UiRect::horizontal(Val::Px(12.0)),
            border: UiRect::all(Val::Px(1.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
        }
        BorderColor {
            top: Color::srgb(0.38, 0.39, 0.40),
            right: Color::srgb(0.38, 0.39, 0.40),
            bottom: Color::srgb(0.38, 0.39, 0.40),
            left: Color::srgb(0.38, 0.39, 0.40),
        }
        BackgroundColor(Color::srgba(0.18, 0.20, 0.22, 0.96))
    }
}

fn settings_dropdown_label_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        TextFont {
            font_size: {default_font_size(14.0)}
        }
        TextColor(Color::WHITE)
    }
}

fn settings_dropdown_caret_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("v")
        TextFont {
            font_size: {default_font_size(12.0)}
        }
        TextColor(Color::srgb(0.72, 0.80, 0.84))
    }
}
