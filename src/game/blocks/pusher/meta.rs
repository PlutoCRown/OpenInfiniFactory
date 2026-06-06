use super::PusherBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb, rgba};

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
            rgb(0.42, 0.44, 0.42),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Blocker)
    }
}
