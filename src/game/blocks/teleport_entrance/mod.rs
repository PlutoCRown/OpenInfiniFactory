use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct TeleportEntranceBlock;

pub static BLOCK: BlockImpl<TeleportEntranceBlock> = BlockImpl(TeleportEntranceBlock);

mod meta;
mod render;
mod ui;

impl crate::game::blocks::traits::BlockBehavior for TeleportEntranceBlock {}

register_block!(BLOCK, BlockKind::TeleportEntrance, editable: true);
