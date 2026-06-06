use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct GeneratorBlock;

pub static BLOCK: BlockImpl<GeneratorBlock> = BlockImpl(GeneratorBlock);

mod meta;
mod behavior;
mod ui;

impl crate::game::blocks::traits::BlockRender for GeneratorBlock {}

register_block!(BLOCK, BlockKind::Generator, editable: true);
