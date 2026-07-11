use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;

pub struct PusherBlock;

pub static BLOCK: BlockImpl<PusherBlock> = BlockImpl(PusherBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Pusher, editable: false, play: true);
