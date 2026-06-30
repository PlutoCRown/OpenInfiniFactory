use super::BlockerBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for BlockerBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Blocker
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.blocker",
            "short.blocker",
            rgb(0.54, 0.56, 0.54),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Pusher)
    }
}

impl PlaceableBlock for BlockerBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.42, 0.44, 0.42).color()
    }
}
