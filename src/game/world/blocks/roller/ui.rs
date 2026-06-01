use super::*;

pub(super) fn default_settings(
    _block: &RollerBlock,
    _pos: bevy::prelude::IVec3,
) -> Option<BlockSettings> {
    Some(BlockSettings::Labeler(LabelerSettings::default()))
}

pub(super) fn ui_panel(_block: &RollerBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Labeler)
}

pub(super) fn handle_edit_action(
    _block: &RollerBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    edit_labeler(ctx, action);
}
