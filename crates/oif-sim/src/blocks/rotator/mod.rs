use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct RotatorBlock;

pub static BLOCK: BlockImpl<RotatorBlock> = BlockImpl(RotatorBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Rotator, editable: false, play: true);
