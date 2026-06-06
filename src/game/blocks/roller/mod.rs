use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct RollerBlock;

pub static BLOCK: BlockImpl<RollerBlock> = BlockImpl(RollerBlock);

mod meta;
mod behavior;
mod render;
mod ui;

register_block!(BLOCK, BlockKind::Roller, editable: true);
