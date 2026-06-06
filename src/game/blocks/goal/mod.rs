use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct GoalBlock;

pub static BLOCK: BlockImpl<GoalBlock> = BlockImpl(GoalBlock);

mod meta;
mod render;
mod ui;

impl crate::game::blocks::traits::BlockBehavior for GoalBlock {}

register_block!(BLOCK, BlockKind::Goal, editable: true);
