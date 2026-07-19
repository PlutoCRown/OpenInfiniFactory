use crate::blocks::adapter::BlockImpl;
use crate::blocks::traits::BlockBehavior;
use crate::blocks::BlockKind;

pub struct GoalBlock;

pub static BLOCK: BlockImpl<GoalBlock> = BlockImpl(GoalBlock);

mod meta;

impl BlockBehavior for GoalBlock {
    fn accepts_material(&self) -> bool {
        true
    }

    fn shows_material_preview(&self) -> bool {
        true
    }
}

register_block!(BLOCK, BlockKind::Goal, editable: true);
