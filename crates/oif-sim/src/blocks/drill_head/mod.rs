use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct DrillHeadBlock;

pub static BLOCK: BlockImpl<DrillHeadBlock> = BlockImpl(DrillHeadBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::DrillHead, editable: false);
