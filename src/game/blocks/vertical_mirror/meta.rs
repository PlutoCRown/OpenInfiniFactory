use super::VerticalMirrorBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{rgb, BlockDefinition, BlockKind};

impl BlockMeta for VerticalMirrorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::VerticalMirror
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.vertical_mirror",
            "short.vertical_mirror",
            rgb(0.45, 0.88, 1.0),
        )
        .transparent()
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Mirror)
    }
}

impl PlaceableBlock for VerticalMirrorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(1.0, 1.0, 1.0).color()
    }
}
