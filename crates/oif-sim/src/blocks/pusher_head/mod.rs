//! 活塞/拦截器伸出头：真实占格（有碰撞），视觉仍在本体模型动画上

use crate::blocks::BlockKind;
use crate::blocks::adapter::BlockImpl;

pub struct PusherHeadBlock;

pub static BLOCK: BlockImpl<PusherHeadBlock> = BlockImpl(PusherHeadBlock);

mod behavior;
mod meta;

register_block!(BLOCK, BlockKind::PusherHead, editable: false);
