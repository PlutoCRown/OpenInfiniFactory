use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, MaterialDestroyer,
    ModelMaterial, ModelMesh, SystemBlock,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Drill, [0.0, 0.0, 0.0]),
    BlockModelPart::new(ModelMesh::RodX, ModelMaterial::Drill, [0.0, 0.0, 0.0])
        .scaled([0.72, 0.42, 0.42]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Drill, [0.0, 0.0, 0.0])
        .scaled([0.42, 0.42, 0.72]),
];

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
        BlockModel::Parts(MODEL)
    }
}

impl SystemBlock for DrillHeadBlock {}
