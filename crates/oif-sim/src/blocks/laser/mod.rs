use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct LaserBlock;

pub static BLOCK: BlockImpl<LaserBlock> = BlockImpl(LaserBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Laser, editable: false, play: true);
