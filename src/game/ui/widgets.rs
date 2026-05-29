use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use super::components::{default_button_size, default_font_size, menu_button};
use super::types::{
    AreaKind, InventoryItem, InventorySlot, KeyBindingButton, KeyBindingLabel, MainMenuAction,
    PauseAction, SaveListAction, SaveListLabel, ScrollContainer, ScrollContent, SettingsAction,
    SettingsDropdown, SettingsDropdownLabel, SettingsDropdownList, SettingsSlider,
    SettingsSliderFill, SettingsSliderKnob, SettingsValue, SettingsValueText, SlotArea, SlotLabel,
};

pub(super) fn spawn_slot(parent: &mut ChildBuilder, area: SlotArea, index: usize) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(default_button_size(54.0)),
                    height: Val::Px(default_button_size(54.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                border_color: Color::srgb(0.22, 0.22, 0.22).into(),
                background_color: Color::srgba(0.28, 0.28, 0.30, 0.92).into(),
                ..default()
            },
            InventorySlot { area, index },
        ))
        .with_children(|slot| {
            slot.spawn((
                TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: default_font_size(12.0),
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_justify(JustifyText::Center),
                    style: Style {
                        margin: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    ..default()
                },
                SlotLabel,
            ));
        });
}

pub(super) fn spawn_localized_pause_button(
    parent: &mut ChildBuilder,
    label: String,
    key: &'static str,
    action: PauseAction,
) {
    spawn_localized_button(parent, 38.0, 16.0, label, key, action);
}

pub(super) fn spawn_generator_button(
    parent: &mut ChildBuilder,
    label: String,
    key: &'static str,
    action: super::types::GeneratorAction,
) {
    parent
        .spawn((menu_button(36.0), action))
        .with_children(|button| {
            button.spawn((
                TextBundle::from_section(
                    label,
                    TextStyle {
                        font_size: default_font_size(14.0),
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                super::types::LocalizedText { key },
            ));
        });
}

pub(super) fn spawn_localized_settings_button(
    parent: &mut ChildBuilder,
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
            TextBundle::from_section(
                label,
                TextStyle {
                    font_size: default_font_size(14.0),
                    color: Color::WHITE,
                    ..default()
                },
            ),
            super::types::LocalizedText { key },
        ));
        if is_binding {
            label_entity.insert(KeyBindingLabel);
        }
    });
}

pub(super) fn spawn_settings_tab(
    parent: &mut ChildBuilder,
    label: String,
    key: &'static str,
    action: SettingsAction,
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
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
                border_color: Color::srgb(0.38, 0.39, 0.40).into(),
                background_color: Color::srgba(0.16, 0.17, 0.18, 0.96).into(),
                ..default()
            },
            action,
        ))
        .with_children(|tab| {
            tab.spawn((
                TextBundle::from_section(
                    label,
                    TextStyle {
                        font_size: default_font_size(15.0),
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                super::types::LocalizedText { key },
            ));
        });
}

pub(super) fn spawn_settings_slider(parent: &mut ChildBuilder, slider: SettingsSlider) {
    parent.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: default_font_size(13.0),
                color: Color::srgb(0.88, 0.94, 0.96),
                ..default()
            },
        ),
        SettingsValueText(match slider {
            SettingsSlider::Fov => SettingsValue::Fov,
            SettingsSlider::UiScale => SettingsValue::UiScale,
        }),
    ));
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(340.0),
                    height: Val::Px(default_button_size(22.0)),
                    padding: UiRect::all(Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_color: Color::srgb(0.42, 0.44, 0.46).into(),
                background_color: Color::srgba(0.10, 0.11, 0.12, 0.98).into(),
                ..default()
            },
            match slider {
                SettingsSlider::Fov => SettingsAction::FovSlider,
                SettingsSlider::UiScale => SettingsAction::UiScaleSlider,
            },
        ))
        .with_children(|track| {
            track.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::srgb(0.32, 0.62, 0.72).into(),
                    ..default()
                },
                SettingsSliderFill(slider),
            ));
            track.spawn((
                NodeBundle {
                    style: Style {
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
                    background_color: Color::srgb(0.90, 0.96, 1.0).into(),
                    ..default()
                },
                SettingsSliderKnob(slider),
            ));
        });
}

pub(super) fn spawn_settings_dropdown(parent: &mut ChildBuilder, dropdown: SettingsDropdown) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(260.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                position_type: PositionType::Relative,
                ..default()
            },
            background_color: Color::NONE.into(),
            z_index: ZIndex::Global(80),
            ..default()
        })
        .with_children(|container| {
            container
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(default_button_size(36.0)),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        border_color: Color::srgb(0.38, 0.39, 0.40).into(),
                        background_color: Color::srgba(0.18, 0.20, 0.22, 0.96).into(),
                        ..default()
                    },
                    SettingsAction::ToggleDropdown(dropdown),
                ))
                .with_children(|button| {
                    button.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font_size: default_font_size(14.0),
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        SettingsDropdownLabel(dropdown),
                    ));
                    button.spawn(TextBundle::from_section(
                        "v",
                        TextStyle {
                            font_size: default_font_size(12.0),
                            color: Color::srgb(0.72, 0.80, 0.84),
                            ..default()
                        },
                    ));
                });
            container
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            top: Val::Px(default_button_size(40.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(3.0),
                            ..default()
                        },
                        background_color: Color::srgba(0.10, 0.11, 0.12, 0.98).into(),
                        z_index: ZIndex::Global(120),
                        ..default()
                    },
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

fn spawn_dropdown_option(parent: &mut ChildBuilder, label: String, action: SettingsAction) {
    parent
        .spawn((menu_button(32.0), action))
        .with_children(|button| {
            button.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: default_font_size(13.0),
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

fn spawn_localized_dropdown_option(
    parent: &mut ChildBuilder,
    key: &'static str,
    action: SettingsAction,
) {
    parent
        .spawn((menu_button(32.0), action))
        .with_children(|button| {
            button.spawn((
                TextBundle::from_section(
                    key,
                    TextStyle {
                        font_size: default_font_size(13.0),
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                super::types::LocalizedText { key },
            ));
        });
}

pub(super) fn spawn_localized_main_button(
    parent: &mut ChildBuilder,
    label: String,
    key: &'static str,
    action: MainMenuAction,
) {
    spawn_localized_button(parent, 44.0, 17.0, label, key, action);
}

pub(super) fn scroll_container(height: f32) -> (NodeBundle, ScrollContainer) {
    (
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(height),
                position_type: PositionType::Relative,
                overflow: Overflow::clip_y(),
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        },
        ScrollContainer {
            offset: 0.0,
            max_offset: 0.0,
        },
    )
}

pub(super) fn scroll_content() -> ScrollContent {
    ScrollContent
}

pub(super) fn spawn_save_slot_button(parent: &mut ChildBuilder, index: usize) {
    parent
        .spawn((menu_button(34.0), SaveListAction::Load(index)))
        .with_children(|button| {
            button.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: default_font_size(15.0),
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                SaveListLabel,
            ));
        });
}

pub(super) fn spawn_save_back_button(parent: &mut ChildBuilder) {
    parent
        .spawn((menu_button(38.0), SaveListAction::Back))
        .with_children(|button| {
            button.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: default_font_size(16.0),
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                SaveListLabel,
            ));
        });
}

fn spawn_localized_button<'a, A: Bundle>(
    parent: &'a mut ChildBuilder,
    height: f32,
    font_size: f32,
    label: String,
    key: &'static str,
    action: A,
) -> EntityCommands<'a> {
    let mut entity = parent.spawn((menu_button(height), action));
    entity.with_children(|button| {
        button.spawn((
            TextBundle::from_section(
                label,
                TextStyle {
                    font_size: default_font_size(font_size),
                    color: Color::WHITE,
                    ..default()
                },
            ),
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
