use bevy::prelude::*;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, MovementRule,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Belt, [0.0, 0.54, 0.0]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::BeltStripe, [-0.24, 0.59, -0.08])
        .scaled([0.52, 0.36, 0.54]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::BeltStripe, [0.24, 0.59, -0.08])
        .scaled([0.52, 0.36, 0.54]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::BeltStripe, [0.0, 0.62, -0.38]),
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
            rgb(0.10, 0.22, 0.28),
            rgb(0.08, 0.20, 0.26),
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
