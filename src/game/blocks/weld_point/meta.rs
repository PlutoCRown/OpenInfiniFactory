use super::WeldPointBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgba};

impl BlockMeta for WeldPointBlock {
    fn id(&self) -> BlockKind {
        BlockKind::WeldPoint
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.weld_point",
            "short.weld_point",
            rgba(1.0, 0.28, 0.18, 0.45),
        )
        .node()
        .transparent()
        .no_collision()
    }
}
