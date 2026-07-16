use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct LifterBlock;

pub static BLOCK: BlockImpl<LifterBlock> = BlockImpl(LifterBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Lifter, editable: false, play: true);
