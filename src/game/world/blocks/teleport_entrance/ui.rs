use super::*;
use crate::game::ui::components::{label_text, menu_button};
use crate::game::ui::types::{BlockPanelText, BlockPanelTextKind, LocalizedText};
use crate::game::world::blocks::panel_layout::{panel_text, spawn_block_panel, spawn_panel_row};
use crate::game::world::blocks::ui_components::{
    spawn_block_panel_dropdown, spawn_block_panel_dropdown_list,
};
use crate::shared::i18n::I18n;
use bevy::prelude::*;

pub(super) fn ui_panel(_block: &TeleportEntranceBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Teleport)
}

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_block_panel(
        root,
        i18n,
        460.0,
        "teleport.title",
        UiPanelId::Teleport,
        |panel| {
            spawn_panel_row(panel, i18n, "panel.name", |row| {
                spawn_teleport_button(row, TeleportAction::Rename, "button.teleport_rename");
                row.spawn((
                    panel_text("", 18.0, Color::WHITE),
                    BlockPanelText(BlockPanelTextKind::TeleportName),
                ));
            });
            spawn_panel_row(panel, i18n, "panel.pair", |row| {
                spawn_block_panel_dropdown(
                    row,
                    BlockPanelDropdown::TeleportPair,
                    TeleportAction::TogglePairDropdown,
                );
            });
        },
    );
}

pub(crate) fn spawn_dropdown_layers(root: &mut ChildSpawnerCommands) {
    spawn_block_panel_dropdown_list(
        root,
        BlockPanelDropdown::TeleportPair,
        std::iter::empty::<(String, TeleportAction)>(),
    );
}

fn spawn_teleport_button(
    parent: &mut ChildSpawnerCommands,
    action: TeleportAction,
    text_key: &'static str,
) {
    parent
        .spawn((menu_button(36.0), action))
        .with_children(|button| {
            button.spawn((
                label_text(text_key, 14.0, Color::WHITE),
                LocalizedText { key: text_key },
            ));
        });
}
