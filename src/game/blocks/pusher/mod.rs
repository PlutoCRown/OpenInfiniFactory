use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct PusherBlock;

pub static BLOCK: BlockImpl<PusherBlock> = BlockImpl(PusherBlock);

mod meta;
mod behavior;
pub mod model;
mod render;

register_block!(BLOCK, BlockKind::Pusher, editable: false, play: true);
