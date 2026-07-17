use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;

/// 印花机实体占格虚拟方块
pub struct StamperBodyBlock;

pub static BLOCK: BlockImpl<StamperBodyBlock> = BlockImpl(StamperBodyBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::StamperBody, editable: false);
