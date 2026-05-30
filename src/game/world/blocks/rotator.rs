use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, MovementRule,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::RotatorBase,
        ModelMaterial::PlatformBase,
        [0.0, 0.0, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RotatorDisk,
        ModelMaterial::ConveyorBelt,
        [0.0, 0.50, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RotatorRing,
        ModelMaterial::Belt,
        [0.0, 0.48, 0.0],
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
        BlockModel::PartsOnly(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::CounterRotator)
    }
}

impl FactoryBlock for RotatorBlock {}
