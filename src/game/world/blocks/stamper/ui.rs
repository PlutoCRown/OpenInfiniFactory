use super::*;

pub(super) fn default_settings(
    _block: &StamperBlock,
    _pos: bevy::prelude::IVec3,
) -> Option<BlockSettings> {
    Some(BlockSettings::Labeler(LabelerSettings::default()))
}

pub(super) fn ui_panel(_block: &StamperBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Labeler)
}

pub(super) fn handle_edit_action(
    _block: &StamperBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    edit_labeler(ctx, action);
}
