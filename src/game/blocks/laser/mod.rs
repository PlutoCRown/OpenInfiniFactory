use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct LaserBlock;

pub static BLOCK: BlockImpl<LaserBlock> = BlockImpl(LaserBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::Laser, editable: false, play: true);
