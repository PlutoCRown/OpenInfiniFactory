use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct DetectorBlock;

pub static BLOCK: BlockImpl<DetectorBlock> = BlockImpl(DetectorBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::Detector, editable: false, play: true);
