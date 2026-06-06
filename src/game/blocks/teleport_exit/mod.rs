use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct TeleportExitBlock;

pub static BLOCK: BlockImpl<TeleportExitBlock> = BlockImpl(TeleportExitBlock);

mod meta;
mod render;
mod ui;

impl crate::game::blocks::traits::BlockBehavior for TeleportExitBlock {}

register_block!(BLOCK, BlockKind::TeleportExit, editable: true);
