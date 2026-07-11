use super::DetectorBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{
    render_directional_wire_device, BlockModel, BlockModelPart, ModelMaterial, ModelMesh,
    RenderBehavior,
};
use crate::game::world::direction::Facing;

// DrillBody 为 1×1×0.8，Z 向缩放到 0.9；中心偏 +Z 0.05，使 -Z 工作面凹进 0.1
const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::DrillBody,
        ModelMaterial::DetectorBody,
        [0.0, 0.0, 0.05],
    )
    .scaled([1.0, 1.0, 1.125]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Power, [0.0, 0.0, -0.4]),
];

impl BlockRender for DetectorBlock {
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        render_directional_wire_device(facing.forward_ivec3())
    }

    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
