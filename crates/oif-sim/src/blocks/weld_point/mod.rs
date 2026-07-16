use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct WeldPointBlock;

pub static BLOCK: BlockImpl<WeldPointBlock> = BlockImpl(WeldPointBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::WeldPoint, editable: false);
