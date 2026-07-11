use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct BlockerBlock;

pub static BLOCK: BlockImpl<BlockerBlock> = BlockImpl(BlockerBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Blocker, editable: false, play: true);
