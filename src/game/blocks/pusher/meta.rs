use super::PusherBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for PusherBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Pusher
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.pusher",
            "short.pusher",
            rgb(0.54, 0.56, 0.54),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Blocker)
    }
}

impl PlaceableBlock for PusherBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.42, 0.44, 0.42).color()
    }
}
