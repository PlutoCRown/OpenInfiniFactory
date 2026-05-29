use bevy::prelude::*;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, MovementRule,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Belt, [0.0, 0.54, 0.0]),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::BeltStripe,
        [-0.13, 0.59, -0.28],
    )
    .scaled([0.46, 0.30, 0.42])
    .yawed(0.7853982),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::BeltStripe,
        [0.13, 0.59, -0.28],
    )
    .scaled([0.46, 0.30, 0.42])
    .yawed(-0.7853982),
];

pub struct ConveyorBlock;

pub static CONVEYOR: ConveyorBlock = ConveyorBlock;

impl Block for ConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Conveyor
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.conveyor",
            "short.conveyor",
            rgb(0.86, 0.46, 0.14),
            rgb(0.70, 0.34, 0.08),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, facing: super::Facing) -> Option<MovementRule> {
        Some(MovementRule::Translate {
            source: IVec3::Y,
            offset: facing.forward_ivec3(),
        })
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::ReverseConveyor)
    }
}

impl FactoryBlock for ConveyorBlock {}
