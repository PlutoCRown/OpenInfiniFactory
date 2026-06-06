use super::ConverterBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb, rgba};
use bevy::prelude::{IVec3};
use crate::game::world::grid::{BlockSettings, ConverterSettings};

impl BlockMeta for ConverterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Converter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.converter",
            "short.converter",
            rgb(0.50, 0.36, 0.78),
            rgb(0.36, 0.24, 0.62),
        )
        .no_collision()
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Converter(ConverterSettings::default()))
    }
}
