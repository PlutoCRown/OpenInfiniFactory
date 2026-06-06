use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct WeldPointBlock;

pub static BLOCK: BlockImpl<WeldPointBlock> = BlockImpl(WeldPointBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::WeldPoint, editable: false);
