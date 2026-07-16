use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct StamperBlock;

pub static BLOCK: BlockImpl<StamperBlock> = BlockImpl(StamperBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Stamper, editable: true);
