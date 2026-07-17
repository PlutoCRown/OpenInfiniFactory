use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;

/// 滚刷机实体占格虚拟方块
pub struct RollerBodyBlock;

pub static BLOCK: BlockImpl<RollerBodyBlock> = BlockImpl(RollerBodyBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::RollerBody, editable: false);
