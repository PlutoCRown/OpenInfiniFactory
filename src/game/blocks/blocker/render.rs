use super::BlockerBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh, RenderBehavior, WireConnectorBehavior};
use crate::game::world::direction::{Facing};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::PusherBody,
        ModelMaterial::StoneTexture,
        [0.0, 0.0, 0.10],
    ),
    BlockModelPart::new(
        ModelMesh::PusherHead,
        ModelMaterial::BorderedWoodTexture,
        [0.0, 0.0, -0.40],
    ),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::StoneTexture,
        [0.0, 0.0, -0.18],
    )
    .scaled([1.35, 1.35, 0.70]),
];

impl BlockRender for BlockerBlock {
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        RenderBehavior {
            wire_connector: Some(WireConnectorBehavior::Device {
                blocked_offset: facing.forward_ivec3(),
            }),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
