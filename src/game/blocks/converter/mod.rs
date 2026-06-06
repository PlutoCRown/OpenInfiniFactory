use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct ConverterBlock;

pub static BLOCK: BlockImpl<ConverterBlock> = BlockImpl(ConverterBlock);

mod meta;
mod render;
mod ui;

impl crate::game::blocks::traits::BlockBehavior for ConverterBlock {}

register_block!(BLOCK, BlockKind::Converter, editable: true);
