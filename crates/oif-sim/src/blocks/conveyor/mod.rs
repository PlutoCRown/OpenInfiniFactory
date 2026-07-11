use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct ConveyorBlock;

pub static BLOCK: BlockImpl<ConveyorBlock> = BlockImpl(ConveyorBlock);

mod behavior;
mod meta;

register_block!(BLOCK, BlockKind::Conveyor, editable: false, play: true);
