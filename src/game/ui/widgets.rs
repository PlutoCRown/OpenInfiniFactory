use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use crate::game::world::blocks::BlockKind;

use super::components::menu_button;
use super::types::{
    InventorySlot, KeyBindingButton, KeyBindingLabel, LanguageText, MainMenuAction, PauseAction,
    SaveListAction, SaveListLabel, SettingsAction, SimulationAction, SlotArea, SlotLabel,
};

pub(super) fn spawn_slot(parent: &mut ChildBuilder, area: SlotArea, index: usize) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(54.0),
                    height: Val::Px(54.0),
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
                            font_size: 12.0,
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

pub(super) fn spawn_language_settings_button(
    parent: &mut ChildBuilder,
    label: impl Into<String>,
    action: SettingsAction,
) {
    parent
        .spawn((menu_button(36.0), action))
        .with_children(|button| {
            button.spawn((
                TextBundle::from_section(
                    label,
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                LanguageText,
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
                    font_size: 14.0,
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

pub(super) fn spawn_localized_sim_button(
    parent: &mut ChildBuilder,
    label: String,
    key: &'static str,
    action: SimulationAction,
) {
    spawn_localized_button(parent, 34.0, 12.0, label, key, action);
}

pub(super) fn spawn_localized_main_button(
    parent: &mut ChildBuilder,
    label: String,
    key: &'static str,
    action: MainMenuAction,
) {
    spawn_localized_button(parent, 44.0, 17.0, label, key, action);
}

pub(super) fn spawn_save_slot_button(parent: &mut ChildBuilder, index: usize) {
    parent
        .spawn((menu_button(34.0), SaveListAction::Load(index)))
        .with_children(|button| {
            button.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 15.0,
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
                        font_size: 16.0,
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
                    font_size,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            super::types::LocalizedText { key },
        ));
    });
    entity
}

pub(super) fn slot_color(kind: BlockKind) -> Color {
    match kind {
        BlockKind::Solid => Color::srgb(0.38, 0.39, 0.40),
        BlockKind::Glass => Color::srgb(0.42, 0.66, 0.76),
        BlockKind::Generator => Color::srgb(0.42, 0.20, 0.56),
        BlockKind::Welder => Color::srgb(0.62, 0.12, 0.12),
        BlockKind::Conveyor => Color::srgb(0.08, 0.20, 0.26),
        BlockKind::Detector => Color::srgb(0.12, 0.34, 0.62),
        BlockKind::Wire => Color::srgb(0.88, 0.62, 0.12),
        BlockKind::Piston => Color::srgb(0.66, 0.43, 0.20),
        BlockKind::Goal => Color::srgb(0.24, 0.56, 0.30),
        BlockKind::Material => Color::srgb(0.74, 0.74, 0.78),
        BlockKind::WeldPoint => Color::srgb(0.86, 0.16, 0.12),
    }
}

pub(super) fn short_item_name(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::Solid => "short.solid",
        BlockKind::Glass => "short.glass",
        BlockKind::Generator => "short.generator",
        BlockKind::Welder => "short.welder",
        BlockKind::Conveyor => "short.conveyor",
        BlockKind::Detector => "short.detector",
        BlockKind::Wire => "short.wire",
        BlockKind::Piston => "short.piston",
        BlockKind::Goal => "short.goal",
        BlockKind::Material => "short.material",
        BlockKind::WeldPoint => "short.weld_point",
    }
}
