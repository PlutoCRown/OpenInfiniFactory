use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct DrillBlock;

pub static BLOCK: BlockImpl<DrillBlock> = BlockImpl(DrillBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::Drill, editable: false, play: true);
