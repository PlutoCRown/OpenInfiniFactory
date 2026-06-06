use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct WireBlock;

pub static BLOCK: BlockImpl<WireBlock> = BlockImpl(WireBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::Wire, editable: false, play: true);
