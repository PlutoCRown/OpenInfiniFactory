use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct DetectorBlock;

pub static BLOCK: BlockImpl<DetectorBlock> = BlockImpl(DetectorBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Detector, editable: false, play: true);
