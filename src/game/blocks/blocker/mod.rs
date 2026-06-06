use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct BlockerBlock;

pub static BLOCK: BlockImpl<BlockerBlock> = BlockImpl(BlockerBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::Blocker, editable: false, play: true);
