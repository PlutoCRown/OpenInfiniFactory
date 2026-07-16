use super::WelderBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for WelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Welder
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.welder",
            "short.welder",
            "desc.welder",
            rgb(0.14, 0.38, 0.74),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::DownWelder)
    }
}

