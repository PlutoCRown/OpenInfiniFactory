use super::StamperBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};
use bevy::prelude::{IVec3};
use crate::game::world::grid::{BlockSettings, LabelerSettings};

impl BlockMeta for StamperBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Stamper
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.stamper",
            "short.stamper",
            rgb(0.82, 0.26, 0.58),
        )
        .no_collision()
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Labeler(LabelerSettings::default()))
    }
}

impl PlaceableBlock for StamperBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.64, 0.14, 0.42).color()
    }
}
