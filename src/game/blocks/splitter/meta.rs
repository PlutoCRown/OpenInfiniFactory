use super::SplitterBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{rgb, BlockDefinition, BlockKind};

impl BlockMeta for SplitterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Splitter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.splitter",
            "short.splitter",
            rgb(0.45, 0.88, 1.0),
        )
        .transparent()
    }
}

impl PlaceableBlock for SplitterBlock {
    fn item_slot_color(&self) -> Color {
        rgb(1.0, 1.0, 1.0).color()
    }
}
