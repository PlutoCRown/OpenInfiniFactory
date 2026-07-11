use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct RollerBlock;

pub static BLOCK: BlockImpl<RollerBlock> = BlockImpl(RollerBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Roller, editable: true);
