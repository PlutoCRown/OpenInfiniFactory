use super::*;

pub(super) fn default_settings(
    _block: &GoalBlock,
    _pos: bevy::prelude::IVec3,
) -> Option<BlockSettings> {
    Some(BlockSettings::Goal(GoalSettings::default()))
}

pub(super) fn ui_panel(_block: &GoalBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Goal)
}

pub(super) fn handle_edit_action(
    _block: &GoalBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    let mut settings = ctx.world.goal_settings(ctx.pos);
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
    ctx.world.set_goal_settings(ctx.pos, settings);
    ctx.mark_dirty();
}
