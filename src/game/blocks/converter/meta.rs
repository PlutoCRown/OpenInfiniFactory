use super::ConverterBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};
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
        )
        .no_collision()
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Converter(ConverterSettings::default()))
    }
}

impl PlaceableBlock for ConverterBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.36, 0.24, 0.62).color()
    }
}
