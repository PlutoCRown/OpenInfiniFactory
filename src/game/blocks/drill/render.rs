use super::DrillBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh, RenderBehavior, WireConnectorBehavior};
use crate::game::world::direction::{Facing};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::DrillBody,
        ModelMaterial::Platform,
        [0.0, 0.0, 0.10],
    ),
    BlockModelPart::new(
        ModelMesh::DrillTip,
        ModelMaterial::DrillTip,
        [0.0, 0.0, -0.34],
    ),
];

impl BlockRender for DrillBlock {
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
