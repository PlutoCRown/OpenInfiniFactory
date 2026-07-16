use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct TeleportExitBlock;

pub static BLOCK: BlockImpl<TeleportExitBlock> = BlockImpl(TeleportExitBlock);

mod meta;

impl crate::blocks::traits::BlockBehavior for TeleportExitBlock {}

register_block!(BLOCK, BlockKind::TeleportExit, editable: true);
