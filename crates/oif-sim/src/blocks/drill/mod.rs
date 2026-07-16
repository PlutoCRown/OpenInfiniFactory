use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct DrillBlock;

pub static BLOCK: BlockImpl<DrillBlock> = BlockImpl(DrillBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Drill, editable: false, play: true);
