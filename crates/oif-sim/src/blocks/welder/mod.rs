use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct WelderBlock;

pub static BLOCK: BlockImpl<WelderBlock> = BlockImpl(WelderBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Welder, editable: false, play: true);
