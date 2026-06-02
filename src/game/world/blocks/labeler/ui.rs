use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::ui::components::{default_button_size, default_font_size, HoverButton};
use crate::game::ui::types::{BlockPanelDropdownLabel, LocalizedText};
use crate::game::ui::{BlockEditAction, BlockPanelDropdown, UiPanelId};
use crate::game::world::blocks::panel_layout::{panel_row_scene, spawn_block_panel};
use crate::game::world::blocks::ui_components::spawn_block_panel_dropdown_list;
use crate::game::world::blocks::StampColor;
use crate::shared::i18n::I18n;

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_block_panel(
        root,
        i18n,
        420.0,
        "labeler.title",
        UiPanelId::Labeler,
        |panel| {
            panel
                .spawn_empty()
                .queue_apply_scene(panel_row_scene())
                .with_children(|row| {
                    spawn_labeler_label(row, "panel.color", i18n);
                    spawn_labeler_color_dropdown(row);
                });
        },
    )
}

fn spawn_labeler_label(row: &mut ChildSpawnerCommands, text_key: &'static str, i18n: &I18n) {
    row.spawn_empty()
        .queue_apply_scene(labeler_label_scene(text_key, i18n));
}

fn labeler_label_scene(text_key: &'static str, i18n: &I18n) -> impl bevy_scene::Scene {
    let text = i18n.text(text_key);
    bsn! {
        Text({text})
        TextFont {
            font_size: {default_font_size(16.0)}
        }
        TextColor(Color::srgb(0.86, 0.88, 0.86))
        LocalizedText {
            key: {text_key}
        }
        Node {
            width: Val::Px(110.0),
        }
    }
}

fn spawn_labeler_color_dropdown(row: &mut ChildSpawnerCommands) {
    row.spawn_empty()
        .queue_apply_scene(labeler_dropdown_container_scene())
        .with_children(|container| {
            container
                .spawn((Button, HoverButton, BlockEditAction::ToggleColorDropdown))
                .queue_apply_scene(labeler_dropdown_button_scene())
                .with_children(|button| {
                    button
                        .spawn(BlockPanelDropdownLabel(BlockPanelDropdown::LabelerColor))
                        .queue_apply_scene(labeler_dropdown_label_scene());
                    button
                        .spawn_empty()
                        .queue_apply_scene(labeler_dropdown_caret_scene());
                });
        });
}

fn labeler_dropdown_container_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Px(230.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            position_type: PositionType::Relative,
        }
        ZIndex(300)
        BackgroundColor(Color::NONE)
    }
}

fn labeler_dropdown_button_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(default_button_size(34.0)),
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

fn labeler_dropdown_label_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        TextFont {
            font_size: {default_font_size(14.0)}
        }
        TextColor(Color::WHITE)
    }
}

fn labeler_dropdown_caret_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("v")
        TextFont {
            font_size: {default_font_size(12.0)}
        }
        TextColor(Color::srgb(0.72, 0.80, 0.84))
    }
}

pub(crate) fn spawn_dropdown_layers(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_block_panel_dropdown_list(
        root,
        BlockPanelDropdown::LabelerColor,
        StampColor::ALL.into_iter().map(|color| {
            (
                i18n.text(color.name_key()),
                BlockEditAction::SetColor(color),
            )
        }),
    )
}
