use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct LifterBlock;

pub static BLOCK: BlockImpl<LifterBlock> = BlockImpl(LifterBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::Lifter, editable: false, play: true);
