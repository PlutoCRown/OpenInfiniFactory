use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui_widgets::{Slider, SliderRange, SliderThumb, SliderValue, TrackClick};

use super::components::{default_button_size, default_font_size, menu_button};
use super::types::{
    AreaKind, InventoryItem, InventorySlot, KeyBindingButton, KeyBindingLabel, MainMenuAction,
    PauseAction, SaveListAction, SaveListLabel, ScrollContainer, ScrollContent, SettingsAction,
    SettingsDropdown, SettingsDropdownLabel, SettingsDropdownList, SettingsSlider,
    SettingsSliderFill, SettingsSliderKnob, SettingsValue, SettingsValueText, SlotArea, SlotLabel,
};

fn label_text(value: impl Into<String>, font_size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: default_font_size(font_size),
            ..default()
        },
        TextColor(color),
    )
}

fn styled_button(style: Node, border: Color, background: Color) -> impl Bundle {
    (Button, style, BorderColor::all(border), BackgroundColor(background))
}

fn plain_node(style: Node) -> impl Bundle {
    (style, BackgroundColor(Color::NONE))
}

pub(super) fn spawn_slot(parent: &mut ChildSpawnerCommands, area: SlotArea, index: usize) {
    parent
        .spawn((
            styled_button(
                Node {
                    width: Val::Px(default_button_size(54.0)),
                    height: Val::Px(default_button_size(54.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Color::srgb(0.22, 0.22, 0.22),
                Color::srgba(0.28, 0.28, 0.30, 0.92),
            ),
            InventorySlot { area, index },
        ))
        .with_children(|slot| {
            slot.spawn((
                label_text("", 12.0, Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                SlotLabel,
            ));
        });
}

pub(super) fn spawn_localized_pause_button(
    parent: &mut ChildSpawnerCommands,
    label: String,
    key: &'static str,
    action: PauseAction,
) {
    spawn_localized_button(parent, 38.0, 16.0, label, key, action);
}

pub(super) fn spawn_generator_button(
    parent: &mut ChildSpawnerCommands,
    label: String,
    key: &'static str,
    action: super::types::GeneratorAction,
) {
    spawn_localized_button(parent, 36.0, 14.0, label, key, action);
}

pub(super) fn spawn_labeler_button(
    parent: &mut ChildSpawnerCommands,
    label: String,
    key: &'static str,
    action: super::types::LabelerAction,
) {
    spawn_localized_button(parent, 36.0, 14.0, label, key, action);
}

pub(super) fn spawn_converter_button(
    parent: &mut ChildSpawnerCommands,
    label: String,
    key: &'static str,
    action: super::types::ConverterAction,
) {
    spawn_localized_button(parent, 36.0, 14.0, label, key, action);
}

pub(super) fn spawn_teleport_button(
    parent: &mut ChildSpawnerCommands,
    label: String,
    key: &'static str,
    action: super::types::TeleportAction,
) {
    spawn_localized_button(parent, 36.0, 14.0, label, key, action);
}

pub(super) fn spawn_localized_settings_button(
    parent: &mut ChildSpawnerCommands,
    label: String,
    key: &'static str,
    action: SettingsAction,
) {
    let is_binding = matches!(action, SettingsAction::Bind(_));
    let mut button = parent.spawn((menu_button(36.0), action));
    if let SettingsAction::Bind(action) = action {
        button.insert(KeyBindingButton(action));
    }
    button.with_children(|button| {
        let mut label_entity = button.spawn((
            label_text(label, 14.0, Color::WHITE),
            super::types::LocalizedText { key },
        ));
        if is_binding {
            label_entity.insert(KeyBindingLabel);
        }
    });
}

pub(super) fn spawn_settings_tab(
    parent: &mut ChildSpawnerCommands,
    label: String,
    key: &'static str,
    action: SettingsAction,
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
                Color::srgb(0.38, 0.39, 0.40),
                Color::srgba(0.16, 0.17, 0.18, 0.96),
            ),
            action,
        ))
        .with_children(|tab| {
            tab.spawn((
                label_text(label, 15.0, Color::WHITE),
                super::types::LocalizedText { key },
            ));
        });
}

pub(super) fn spawn_settings_slider(parent: &mut ChildSpawnerCommands, slider: SettingsSlider) {
    parent.spawn((
        label_text("", 13.0, Color::srgb(0.88, 0.94, 0.96)),
        SettingsValueText(match slider {
            SettingsSlider::Fov => SettingsValue::Fov,
            SettingsSlider::UiScale => SettingsValue::UiScale,
            SettingsSlider::Gravity => SettingsValue::Gravity,
        }),
    ));
    parent
        .spawn((
            styled_button(
                Node {
                    width: Val::Px(340.0),
                    height: Val::Px(default_button_size(22.0)),
                    padding: UiRect::all(Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                Color::srgb(0.42, 0.44, 0.46),
                Color::srgba(0.10, 0.11, 0.12, 0.98),
            ),
            Slider {
                track_click: TrackClick::Snap,
            },
            SliderValue(settings_slider_initial_value(slider)),
            SliderRange::new(0.0, 100.0),
            match slider {
                SettingsSlider::Fov => SettingsAction::FovSlider,
                SettingsSlider::UiScale => SettingsAction::UiScaleSlider,
                SettingsSlider::Gravity => SettingsAction::GravitySlider,
            },
        ))
        .with_children(|track| {
            track.spawn((
                (
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.32, 0.62, 0.72)),
                ),
                SettingsSliderFill(slider),
            ));
            track.spawn((
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Px(3.0),
                        bottom: Val::Px(3.0),
                        width: Val::Px(6.0),
                        margin: UiRect {
                            left: Val::Px(-3.0),
                            ..default()
                        },
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.90, 0.96, 1.0)),
                ),
                SettingsSliderKnob(slider),
                SliderThumb,
            ));
        });
}

fn settings_slider_initial_value(slider: SettingsSlider) -> f32 {
    match slider {
        SettingsSlider::Fov => 50.0,
        SettingsSlider::UiScale => 0.0,
        SettingsSlider::Gravity => 20.0,
    }
}

pub(super) fn spawn_settings_dropdown(parent: &mut ChildSpawnerCommands, dropdown: SettingsDropdown) {
    parent
        .spawn((
            plain_node(Node {
                width: Val::Px(260.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                position_type: PositionType::Relative,
                ..default()
            }),
            ZIndex(80),
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
                    button.spawn((label_text("", 14.0, Color::WHITE), SettingsDropdownLabel(dropdown)));
                    button.spawn(label_text("v", 12.0, Color::srgb(0.72, 0.80, 0.84)));
                });
            container
                .spawn((
                    (
                        Node {
                            width: Val::Percent(100.0),
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            top: Val::Px(default_button_size(40.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(3.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98)),
                        ZIndex(120),
                    ),
                    SettingsDropdownList(dropdown),
                ))
                .with_children(|list| match dropdown {
                    SettingsDropdown::Language => {
                        for language in crate::shared::i18n::Language::ALL {
                            spawn_dropdown_option(
                                list,
                                language.native_name().to_string(),
                                SettingsAction::SetLanguage(language),
                            );
                        }
                    }
                    SettingsDropdown::PlaceSelectionMode => {
                        for mode in crate::shared::config::ConfigSelectionMode::ALL {
                            spawn_localized_dropdown_option(
                                list,
                                mode.label_key(),
                                SettingsAction::SetPlaceSelectionMode(mode),
                            );
                        }
                    }
                    SettingsDropdown::DeleteSelectionMode => {
                        for mode in crate::shared::config::ConfigSelectionMode::ALL {
                            spawn_localized_dropdown_option(
                                list,
                                mode.label_key(),
                                SettingsAction::SetDeleteSelectionMode(mode),
                            );
                        }
                    }
                });
        });
}

fn spawn_dropdown_option(parent: &mut ChildSpawnerCommands, label: String, action: SettingsAction) {
    parent
        .spawn((menu_button(32.0), action))
        .with_children(|button| {
            button.spawn(label_text(label, 13.0, Color::WHITE));
        });
}

fn spawn_localized_dropdown_option(
    parent: &mut ChildSpawnerCommands,
    key: &'static str,
    action: SettingsAction,
) {
    parent
        .spawn((menu_button(32.0), action))
        .with_children(|button| {
            button.spawn((
                label_text(key, 13.0, Color::WHITE),
                super::types::LocalizedText { key },
            ));
        });
}

pub(super) fn spawn_localized_main_button(
    parent: &mut ChildSpawnerCommands,
    label: String,
    key: &'static str,
    action: MainMenuAction,
) {
    spawn_localized_button(parent, 44.0, 17.0, label, key, action);
}

pub(super) fn scroll_container(height: f32) -> (impl Bundle, ScrollContainer) {
    (
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(height),
                position_type: PositionType::Relative,
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ),
        ScrollContainer {
            offset: 0.0,
            max_offset: 0.0,
        },
    )
}

pub(super) fn scroll_content() -> ScrollContent {
    ScrollContent
}

pub(super) fn spawn_save_slot_button(parent: &mut ChildSpawnerCommands, action: SaveListAction) {
    parent
        .spawn((menu_button(34.0), action))
        .with_children(|button| {
            button.spawn((label_text("", 15.0, Color::WHITE), SaveListLabel));
        });
}

pub(super) fn spawn_save_row_button(
    parent: &mut ChildSpawnerCommands,
    action: SaveListAction,
    width: f32,
) {
    parent
        .spawn((
            styled_button(
                Node {
                    width: Val::Px(default_button_size(width)),
                    min_width: Val::Px(default_button_size(width)),
                    height: Val::Px(default_button_size(30.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                super::components::BUTTON_BORDER,
                super::components::BUTTON_BG,
            ),
            action,
        ))
        .with_children(|button| {
            button.spawn((label_text("", 13.0, Color::WHITE), SaveListLabel));
        });
}

pub(super) fn spawn_save_back_button(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((menu_button(38.0), SaveListAction::Back))
        .with_children(|button| {
            button.spawn((label_text("", 16.0, Color::WHITE), SaveListLabel));
        });
}

fn spawn_localized_button<'a, A: Bundle>(
    parent: &'a mut ChildSpawnerCommands,
    height: f32,
    font_size: f32,
    label: String,
    key: &'static str,
    action: A,
) -> EntityCommands<'a> {
    let mut entity = parent.spawn((menu_button(height), action));
    entity.with_children(|button| {
        button.spawn((
            label_text(label, font_size, Color::WHITE),
            super::types::LocalizedText { key },
        ));
    });
    entity
}

pub(super) fn slot_color(item: InventoryItem) -> Color {
    match item {
        InventoryItem::Area(AreaKind::Selection) => Color::srgb(0.22, 0.66, 0.62),
        InventoryItem::Block(kind) => kind.slot_color(),
    }
}

pub(super) fn short_item_name(item: InventoryItem) -> &'static str {
    match item {
        InventoryItem::Area(AreaKind::Selection) => "short.area.selection",
        InventoryItem::Block(kind) => kind.short_name_key(),
    }
}
