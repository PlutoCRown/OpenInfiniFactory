use super::{rgb, Block, BlockDefinition, BlockKind, SystemBlock};

pub struct DrillHeadBlock;

pub static DRILL_HEAD: DrillHeadBlock = DrillHeadBlock;

impl Block for DrillHeadBlock {
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

    fn is_drill_head(&self) -> bool {
        true
    }
}

impl SystemBlock for DrillHeadBlock {}
