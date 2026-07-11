use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;
pub struct GeneratorBlock;

pub static BLOCK: BlockImpl<GeneratorBlock> = BlockImpl(GeneratorBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Generator, editable: true);
