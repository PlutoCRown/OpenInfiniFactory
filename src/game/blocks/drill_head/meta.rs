use super::DrillHeadBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for DrillHeadBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DrillHead
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.drill_head",
            "short.drill_head",
            rgb(0.12, 0.14, 0.16),
            rgb(0.10, 0.11, 0.12),
        )
        .node()
        .transparent()
        .no_collision()
    }
}
