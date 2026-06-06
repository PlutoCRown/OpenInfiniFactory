use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct CounterRotatorBlock;

pub static BLOCK: BlockImpl<CounterRotatorBlock> = BlockImpl(CounterRotatorBlock);

mod meta;
mod behavior;
mod render;

register_block!(BLOCK, BlockKind::CounterRotator, editable: false, play: true);
