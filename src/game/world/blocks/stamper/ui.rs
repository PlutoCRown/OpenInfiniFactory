use super::*;

pub(super) fn handle_edit_action(
    _block: &StamperBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    edit_labeler(ctx, action);
}
