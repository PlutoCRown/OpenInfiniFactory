use super::MirrorBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{rgb, BlockDefinition, BlockKind};

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

