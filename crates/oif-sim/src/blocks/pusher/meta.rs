use super::PusherBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

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

