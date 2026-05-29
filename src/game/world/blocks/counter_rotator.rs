use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, MovementRule,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Rotation, [0.0, 0.54, 0.0]),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::Rotation,
        [0.18, 0.62, -0.14],
    )
    .scaled([0.68, 0.55, 0.55]),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::Rotation,
        [-0.18, 0.62, 0.14],
    )
    .scaled([0.55, 0.55, 0.68]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::Rotation,
        [-0.18, 0.64, -0.34],
    ),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::Rotation,
        [0.18, 0.64, 0.34],
    ),
];

pub struct CounterRotatorBlock;

pub static COUNTER_ROTATOR: CounterRotatorBlock = CounterRotatorBlock;

impl Block for CounterRotatorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::CounterRotator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.counter_rotator",
            "short.counter_rotator",
            rgb(0.62, 0.28, 0.78),
            rgb(0.54, 0.22, 0.68),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: super::Facing) -> Option<MovementRule> {
        Some(MovementRule::Rotate { clockwise: false })
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Rotator)
    }
}

impl FactoryBlock for CounterRotatorBlock {}
