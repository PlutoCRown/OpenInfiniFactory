use super::*;

pub(super) fn handle_edit_action(
    _block: &RollerBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    edit_labeler(ctx, action);
}
