use super::WireBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb, rgba};

impl BlockMeta for WireBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Wire
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.wire",
            "short.wire",
            rgb(0.95, 0.72, 0.18),
            rgb(0.88, 0.62, 0.12),
        )
        .node()
    }
}
