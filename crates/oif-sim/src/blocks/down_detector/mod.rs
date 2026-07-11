use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct DownDetectorBlock;

pub static BLOCK: BlockImpl<DownDetectorBlock> = BlockImpl(DownDetectorBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::DownDetector, editable: false, play: true);
