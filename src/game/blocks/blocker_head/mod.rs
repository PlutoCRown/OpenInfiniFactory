use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct BlockerHeadBlock;

pub static BLOCK: BlockImpl<BlockerHeadBlock> = BlockImpl(BlockerHeadBlock);

mod meta;
mod render;

impl crate::game::blocks::traits::BlockBehavior for BlockerHeadBlock {}

register_block!(BLOCK, BlockKind::BlockerHead, editable: false);
