use super::BlockerBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb, rgba};

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
            rgb(0.42, 0.44, 0.42),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Pusher)
    }
}
