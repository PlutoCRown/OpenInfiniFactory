use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct WireBlock;

pub static BLOCK: BlockImpl<WireBlock> = BlockImpl(WireBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Wire, editable: false, play: true);
