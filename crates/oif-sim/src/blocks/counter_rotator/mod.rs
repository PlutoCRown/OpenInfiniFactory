use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct CounterRotatorBlock;

pub static BLOCK: BlockImpl<CounterRotatorBlock> = BlockImpl(CounterRotatorBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::CounterRotator, editable: false, play: true);
