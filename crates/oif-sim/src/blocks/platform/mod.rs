use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;

pub struct PlatformBlock;

pub static BLOCK: BlockImpl<PlatformBlock> = BlockImpl(PlatformBlock);

mod meta;

impl crate::blocks::traits::BlockBehavior for PlatformBlock {}

register_block!(BLOCK, BlockKind::Platform, editable: false, play: true);
