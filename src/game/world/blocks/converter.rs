use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, SystemBlock};
use crate::game::world::grid::{BlockSettings, ConverterSettings};

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

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Converter(ConverterSettings::default()))
    }
}

impl SystemBlock for ConverterBlock {}
impl EditableBlock for ConverterBlock {}
