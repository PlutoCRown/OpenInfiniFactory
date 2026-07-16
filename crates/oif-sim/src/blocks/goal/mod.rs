use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct GoalBlock;

pub static BLOCK: BlockImpl<GoalBlock> = BlockImpl(GoalBlock);

mod meta;

impl crate::blocks::traits::BlockBehavior for GoalBlock {}
register_block!(BLOCK, BlockKind::Goal, editable: true);
