use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct ReverseConveyorBlock;

pub static BLOCK: BlockImpl<ReverseConveyorBlock> = BlockImpl(ReverseConveyorBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::ReverseConveyor, editable: false, play: true);
