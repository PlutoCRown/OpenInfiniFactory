use super::DownDetectorBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh, RenderBehavior, WireConnectorBehavior};
use bevy::prelude::{IVec3};
use crate::game::world::direction::{Facing};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Signal, [0.0, 0.30, 0.0]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Signal, [0.0, -0.10, 0.0])
        .scaled([0.76, 0.62, 0.76]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Power, [0.0, -0.50, 0.0]),
];

impl BlockRender for DownDetectorBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            wire_connector: Some(WireConnectorBehavior::Device {
                blocked_offset: IVec3::NEG_Y,
            }),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
