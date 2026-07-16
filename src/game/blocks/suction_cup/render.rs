use super::SuctionCupBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{
    render_directional_wire_device, BlockModel, BlockModelPart, ModelMaterial, ModelMesh,
    RenderBehavior,
};
use crate::game::world::direction::Facing;

// 四棱锥：工作面在局部 -Z，顶点在格子中心
const MODEL: &[BlockModelPart] = &[BlockModelPart::new(
    ModelMesh::SuctionCup,
    ModelMaterial::SuctionCup,
    [0.0, 0.0, 0.0],
)];

impl BlockRender for SuctionCupBlock {
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        render_directional_wire_device(facing.forward_ivec3())
    }

    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
