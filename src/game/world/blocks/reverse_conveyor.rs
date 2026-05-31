use bevy::prelude::*;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, ModelMaterial, ModelMesh,
    MovementRule,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::ConveyorBase,
        ModelMaterial::Belt,
        [0.0, 0.0, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::ConveyorBelt,
        ModelMaterial::ConveyorBelt,
        [0.0, -0.50, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::BeltStripe,
        [-0.11, -0.56, 0.26],
    )
    .scaled([0.62, 0.16, 0.42])
    .yawed(-0.7853982),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::BeltStripe,
        [0.11, -0.56, 0.26],
    )
    .scaled([0.62, 0.16, 0.42])
    .yawed(0.7853982),
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
            rgb(0.86, 0.46, 0.14),
            rgb(0.70, 0.34, 0.08),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, facing: super::Facing) -> Option<MovementRule> {
        Some(MovementRule::Translate {
            source: IVec3::NEG_Y,
            offset: facing.forward_ivec3(),
        })
    }

    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Conveyor)
    }
}
