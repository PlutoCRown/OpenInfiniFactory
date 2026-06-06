use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct ConveyorBlock;

pub static BLOCK: BlockImpl<ConveyorBlock> = BlockImpl(ConveyorBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::Conveyor, editable: false, play: true);
