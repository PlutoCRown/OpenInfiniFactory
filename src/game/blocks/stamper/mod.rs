use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct StamperBlock;

pub static BLOCK: BlockImpl<StamperBlock> = BlockImpl(StamperBlock);

mod meta;
mod behavior;
mod render;
mod ui;

register_block!(BLOCK, BlockKind::Stamper, editable: true);
