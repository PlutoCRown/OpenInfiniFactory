use bevy::prelude::*;

use crate::game::ui::{BlockEditAction, BlockPanelDropdown, UiPanelId};
use crate::game::world::blocks::panel_layout::{spawn_block_panel, spawn_panel_row};
use crate::game::world::blocks::ui_components::{
    spawn_block_panel_dropdown, spawn_block_panel_dropdown_list,
};
use crate::game::world::blocks::StampColor;
use crate::shared::i18n::I18n;

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_block_panel(
        root,
        i18n,
        420.0,
        "labeler.title",
        UiPanelId::Labeler,
        |panel| {
            spawn_panel_row(panel, i18n, "panel.color", |row| {
                spawn_block_panel_dropdown(
                    row,
                    BlockPanelDropdown::LabelerColor,
                    BlockEditAction::ToggleColorDropdown,
                );
            });
        },
    );
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
    );
}
