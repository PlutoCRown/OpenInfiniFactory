use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct DownWelderBlock;

pub static BLOCK: BlockImpl<DownWelderBlock> = BlockImpl(DownWelderBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::DownWelder, editable: false, play: true);
