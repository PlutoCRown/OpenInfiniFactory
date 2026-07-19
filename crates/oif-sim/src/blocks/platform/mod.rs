use crate::blocks::adapter::BlockImpl;
use crate::blocks::traits::BlockBehavior;
use crate::blocks::BlockKind;

pub struct PlatformBlock;

pub static BLOCK: BlockImpl<PlatformBlock> = BlockImpl(PlatformBlock);

mod meta;

impl BlockBehavior for PlatformBlock {
    fn is_detector_target(&self) -> bool {
        true
    }
}

register_block!(BLOCK, BlockKind::Platform, editable: false, play: true);
