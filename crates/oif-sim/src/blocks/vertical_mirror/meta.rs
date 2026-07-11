use super::VerticalMirrorBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{rgb, BlockDefinition, BlockKind};

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

