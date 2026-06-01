use super::*;
use crate::game::world::blocks::panel_layout::{spawn_block_panel, spawn_panel_row};
use crate::game::world::blocks::ui_components::{
    spawn_material_icon_dropdown_list, spawn_material_icon_slot,
};
use crate::game::world::blocks::{goal_settings, set_goal_settings, MaterialKind};
use crate::shared::i18n::I18n;
use bevy::prelude::*;

pub(super) fn ui_panel(_block: &GoalBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Goal)
}

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_block_panel(root, i18n, 430.0, "goal.title", UiPanelId::Goal, |panel| {
        spawn_panel_row(panel, i18n, "panel.material", |row| {
            spawn_material_icon_slot(
                row,
                BlockPanelDropdown::GoalMaterial,
                BlockEditAction::ToggleMaterialDropdown,
            );
        });
    });
}

pub(crate) fn spawn_dropdown_layers(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::GoalMaterial,
        MaterialKind::ALL
            .into_iter()
            .map(|material| (material, BlockEditAction::SetMaterial(material))),
    );
}

pub(super) fn handle_edit_action(
    _block: &GoalBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    let mut settings = goal_settings(ctx.world, ctx.pos);
    match action {
        BlockEditAction::ToggleMaterialDropdown => {
            ctx.toggle_dropdown(BlockPanelDropdown::GoalMaterial);
            return;
        }
        BlockEditAction::SetMaterial(material) => {
            settings.material = material;
            ctx.close_dropdown();
        }
        _ => return,
    }
    set_goal_settings(ctx.world, ctx.pos, settings);
    ctx.mark_dirty();
}
