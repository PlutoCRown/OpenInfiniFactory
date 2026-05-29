use bevy::prelude::*;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, MovementRule,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Belt, [0.0, -0.54, 0.0]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::BeltStripe, [-0.24, -0.59, -0.08])
        .scaled([0.52, 0.36, 0.54]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::BeltStripe, [0.24, -0.59, -0.08])
        .scaled([0.52, 0.36, 0.54]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::BeltStripe, [0.0, -0.62, -0.38]),
];

pub struct ReverseConveyorBlock;

pub static REVERSE_CONVEYOR: ReverseConveyorBlock = ReverseConveyorBlock;

impl Block for ReverseConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::ReverseConveyor
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.reverse_conveyor",
            "short.reverse_conveyor",
            rgb(0.14, 0.30, 0.36),
            rgb(0.10, 0.26, 0.32),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, facing: super::Facing) -> Option<MovementRule> {
        Some(MovementRule::Translate {
            source: IVec3::NEG_Y,
            offset: -facing.forward_ivec3(),
        })
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Conveyor)
    }
}

impl FactoryBlock for ReverseConveyorBlock {}
