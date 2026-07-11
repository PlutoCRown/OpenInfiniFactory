use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;

pub struct SuctionCupBlock;

pub static BLOCK: BlockImpl<SuctionCupBlock> = BlockImpl(SuctionCupBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::SuctionCup, editable: false, play: true);
