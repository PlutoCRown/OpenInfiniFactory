use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel, MaterialDestroyer, SystemBlock};

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

    fn material_destroyer(&self, _facing: super::Facing) -> Option<MaterialDestroyer> {
        Some(MaterialDestroyer::AdjacentDrillHead)
    }

    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(&[])
    }
}

impl SystemBlock for DrillHeadBlock {}
