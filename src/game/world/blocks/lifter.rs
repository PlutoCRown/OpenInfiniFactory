use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, MovementRule,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Lift, [0.0, 0.54, 0.0]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [-0.24, 0.18, -0.24]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [0.24, 0.18, -0.24]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [-0.24, 0.18, 0.24]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [0.24, 0.18, 0.24]),
];

pub struct LifterBlock;

pub static LIFTER: LifterBlock = LifterBlock;

impl Block for LifterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Lifter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.lifter",
            "short.lifter",
            rgb(0.25, 0.58, 0.72),
            rgb(0.18, 0.48, 0.62),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: super::Facing) -> Option<MovementRule> {
        Some(MovementRule::Lift { range: 5 })
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}

impl FactoryBlock for LifterBlock {}
