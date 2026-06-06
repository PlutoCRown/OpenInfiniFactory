use super::*;
use crate::game::ui::components::{
    default_button_size, default_font_size, raised_border, HoverButton, BUTTON_BG,
};
use crate::game::ui::types::{
    BlockPanelDropdownLabel, BlockPanelText, BlockPanelTextKind, LocalizedText,
};
use crate::game::ui::{BlockPanelDropdown, UiPanelId};
use crate::game::world::blocks::panel_layout::{panel_row_scene, spawn_block_panel};
use crate::game::world::blocks::ui_components::spawn_block_panel_dropdown_list;
use crate::shared::i18n::I18n;
use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_block_panel(
        root,
        i18n,
        460.0,
        "teleport.title",
        UiPanelId::Teleport,
        |panel| {
            panel
                .spawn(Visibility::Visible)
                .queue_apply_scene(panel_row_scene())
                .with_children(|row| {
                    spawn_teleport_label(row, "panel.name", i18n);
                    spawn_teleport_button(row, TeleportAction::Rename, "button.teleport_rename");
                    row.spawn(BlockPanelText {
                        kind: BlockPanelTextKind::TeleportName,
                    })
                    .queue_apply_scene(teleport_name_text_scene());
                });
            panel
                .spawn(Visibility::Visible)
                .queue_apply_scene(panel_row_scene())
                .with_children(|row| {
                    spawn_teleport_label(row, "panel.pair", i18n);
                    spawn_teleport_pair_dropdown(row);
                });
        },
    )
}

fn spawn_teleport_label(row: &mut ChildSpawnerCommands, text_key: &'static str, i18n: &I18n) {
    row.spawn(Visibility::Visible)
        .queue_apply_scene(teleport_label_scene(text_key, i18n));
}

fn teleport_label_scene(text_key: &'static str, i18n: &I18n) -> impl bevy_scene::Scene {
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

fn teleport_name_text_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        TextFont {
            font_size: {default_font_size(18.0)}
        }
        TextColor(Color::WHITE)
    }
}

fn spawn_teleport_pair_dropdown(row: &mut ChildSpawnerCommands) {
    row.spawn(Visibility::Visible)
        .queue_apply_scene(teleport_dropdown_container_scene())
        .with_children(|container| {
            container
                .spawn((Button, HoverButton, TeleportAction::TogglePairDropdown))
                .queue_apply_scene(teleport_dropdown_button_scene())
                .with_children(|button| {
                    button
                        .spawn(BlockPanelDropdownLabel(BlockPanelDropdown::TeleportPair))
                        .queue_apply_scene(teleport_dropdown_label_scene());
                    button
                        .spawn(Visibility::Visible)
                        .queue_apply_scene(teleport_dropdown_caret_scene());
                });
        });
}

fn teleport_dropdown_container_scene() -> impl bevy_scene::Scene {
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

fn teleport_dropdown_button_scene() -> impl bevy_scene::Scene {
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

fn teleport_dropdown_label_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        TextFont {
            font_size: {default_font_size(14.0)}
        }
        TextColor(Color::WHITE)
    }
}

fn teleport_dropdown_caret_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("v")
        TextFont {
            font_size: {default_font_size(12.0)}
        }
        TextColor(Color::srgb(0.72, 0.80, 0.84))
    }
}

pub(crate) fn spawn_dropdown_layers(root: &mut ChildSpawnerCommands) {
    spawn_block_panel_dropdown_list(
        root,
        BlockPanelDropdown::TeleportPair,
        std::iter::empty::<(String, TeleportAction)>(),
    )
}

fn spawn_teleport_button(
    parent: &mut ChildSpawnerCommands,
    action: TeleportAction,
    text_key: &'static str,
) {
    parent
        .spawn((Button, HoverButton, action))
        .queue_apply_scene(teleport_button_visual_scene())
        .queue_spawn_related_scenes::<Children>(teleport_button_label_scene(text_key));
}

fn teleport_button_visual_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            height: Val::Px(default_button_size(36.0)),
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

fn teleport_button_label_scene(text_key: &'static str) -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text({text_key})
            TextFont {
                font_size: {default_font_size(14.0)}
            }
            TextColor(Color::WHITE)
            LocalizedText {
                key: {text_key}
            }
        )
    }
}
