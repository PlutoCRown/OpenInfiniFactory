use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct WelderBlock;

pub static BLOCK: BlockImpl<WelderBlock> = BlockImpl(WelderBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::Welder, editable: false, play: true);
