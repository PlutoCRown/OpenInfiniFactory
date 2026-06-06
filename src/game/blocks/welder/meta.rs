use super::WelderBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for WelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Welder
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.welder",
            "short.welder",
            rgb(0.14, 0.38, 0.74),
            rgb(0.08, 0.26, 0.58),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::DownWelder)
    }
}
