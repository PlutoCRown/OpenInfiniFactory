use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct DrillHeadBlock;

pub static BLOCK: BlockImpl<DrillHeadBlock> = BlockImpl(DrillHeadBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::DrillHead, editable: false);
