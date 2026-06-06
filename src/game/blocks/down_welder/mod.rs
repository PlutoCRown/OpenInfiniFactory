use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct DownWelderBlock;

pub static BLOCK: BlockImpl<DownWelderBlock> = BlockImpl(DownWelderBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::DownWelder, editable: false, play: true);
