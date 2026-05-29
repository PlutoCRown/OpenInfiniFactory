use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, MovementRule,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Rotation, [0.0, 0.54, 0.0]),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::Rotation,
        [-0.18, 0.62, -0.14],
    )
    .scaled([0.68, 0.55, 0.55]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Rotation, [0.18, 0.62, 0.14])
        .scaled([0.55, 0.55, 0.68]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::Rotation,
        [0.18, 0.64, -0.34],
    ),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::Rotation,
        [-0.18, 0.64, 0.34],
    ),
];

pub struct RotatorBlock;

pub static ROTATOR: RotatorBlock = RotatorBlock;

impl Block for RotatorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Rotator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.rotator",
            "short.rotator",
            rgb(0.48, 0.32, 0.72),
            rgb(0.42, 0.26, 0.64),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: super::Facing) -> Option<MovementRule> {
        Some(MovementRule::Rotate { clockwise: true })
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::CounterRotator)
    }
}

impl FactoryBlock for RotatorBlock {}
