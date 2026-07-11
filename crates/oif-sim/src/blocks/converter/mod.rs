use crate::blocks::adapter::BlockImpl;
use crate::blocks::traits::BlockBehavior;
use crate::blocks::{BlockKind, MaterialProcessor};

pub struct ConverterBlock;

pub static BLOCK: BlockImpl<ConverterBlock> = BlockImpl(ConverterBlock);

mod meta;

impl BlockBehavior for ConverterBlock {
    fn material_processor(&self) -> Option<MaterialProcessor> {
        Some(MaterialProcessor::Converter)
    }
}

register_block!(BLOCK, BlockKind::Converter, editable: true);
