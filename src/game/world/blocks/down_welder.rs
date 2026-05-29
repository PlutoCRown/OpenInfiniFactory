use bevy::prelude::*;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, Facing, FactoryBlock,
    MarkerBehavior, ModelMaterial, ModelMesh, RenderBehavior, WeldConnectorBehavior,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Frame, [0.0, 0.32, 0.0]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Welding, [0.0, -0.08, 0.0])
        .scaled([0.85, 0.62, 0.85]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Welding, [0.0, -0.50, 0.0]),
];

pub struct DownWelderBlock;

pub static DOWN_WELDER: DownWelderBlock = DownWelderBlock;

impl Block for DownWelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownWelder
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.down_welder",
            "short.down_welder",
            rgb(0.14, 0.38, 0.74),
            rgb(0.08, 0.26, 0.58),
        )
    }

    fn marker_behavior(&self, _facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::WeldPoint {
            offset: IVec3::NEG_Y,
            facing: Facing::North,
        })
    }

    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::Offset(IVec3::NEG_Y)),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Welder)
    }
}

impl FactoryBlock for DownWelderBlock {}
