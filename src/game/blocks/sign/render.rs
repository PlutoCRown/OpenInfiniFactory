use super::SignBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

// 竖板贴靠局部 +Z（宿主在背后）；Facing 前向为局部 -Z（文字面朝外）
const MODEL: &[BlockModelPart] = &[BlockModelPart::new(
    ModelMesh::SignBoard,
    ModelMaterial::Wood,
    [0.0, 0.0, 0.47],
)];

impl BlockRender for SignBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
