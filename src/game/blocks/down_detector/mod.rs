use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct DownDetectorBlock;

pub static BLOCK: BlockImpl<DownDetectorBlock> = BlockImpl(DownDetectorBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::DownDetector, editable: false, play: true);
