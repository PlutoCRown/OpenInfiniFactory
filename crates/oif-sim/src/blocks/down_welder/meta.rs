use super::DownWelderBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for DownWelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownWelder
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.down_welder",
            "short.down_welder",
            "desc.down_welder",
            rgb(0.14, 0.38, 0.74),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Welder)
    }
}

