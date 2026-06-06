use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct RotatorBlock;

pub static BLOCK: BlockImpl<RotatorBlock> = BlockImpl(RotatorBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::Rotator, editable: false, play: true);
