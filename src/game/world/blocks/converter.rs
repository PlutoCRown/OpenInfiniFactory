use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, SystemBlock};

pub struct ConverterBlock;

pub static CONVERTER: ConverterBlock = ConverterBlock;

impl Block for ConverterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Converter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.converter",
            "short.converter",
            rgb(0.50, 0.36, 0.78),
            rgb(0.36, 0.24, 0.62),
        )
        .no_collision()
    }
}

impl SystemBlock for ConverterBlock {}
impl EditableBlock for ConverterBlock {}
