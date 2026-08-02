pub use oif_sim::blocks::pusher_head::PusherHeadBlock;

use crate::game::blocks::BlockKind;
use crate::game::blocks::adapter::BlockImpl;

pub static BLOCK: BlockImpl<PusherHeadBlock> = BlockImpl(PusherHeadBlock);

mod render;

register_block!(BLOCK, BlockKind::PusherHead, editable: false);
