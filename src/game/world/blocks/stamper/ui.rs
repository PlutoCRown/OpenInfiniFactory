use super::*;

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
