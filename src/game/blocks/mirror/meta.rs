use super::MirrorBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{rgb, BlockDefinition, BlockKind};

impl BlockMeta for MirrorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Mirror
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.mirror",
            "short.mirror",
            rgb(0.45, 0.88, 1.0),
        )
        .transparent()
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::VerticalMirror)
    }
}

impl PlaceableBlock for MirrorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(1.0, 1.0, 1.0).color()
    }
}
