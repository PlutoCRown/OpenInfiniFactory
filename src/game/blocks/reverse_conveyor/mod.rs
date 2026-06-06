use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct ReverseConveyorBlock;

pub static BLOCK: BlockImpl<ReverseConveyorBlock> = BlockImpl(ReverseConveyorBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::ReverseConveyor, editable: false, play: true);
