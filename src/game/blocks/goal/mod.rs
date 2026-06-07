use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct GoalBlock;

pub static BLOCK: BlockImpl<GoalBlock> = BlockImpl(GoalBlock);

mod meta;
mod ui;

impl crate::game::blocks::traits::BlockBehavior for GoalBlock {}
impl crate::game::blocks::traits::BlockRender for GoalBlock {}

register_block!(BLOCK, BlockKind::Goal, editable: true);
