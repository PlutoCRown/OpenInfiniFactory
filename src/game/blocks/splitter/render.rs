use super::SplitterBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

// 过中心六边形；-180° yaw 已烘焙进 SplitterFace 网格坐标
const MODEL: &[BlockModelPart] = &[BlockModelPart::new(
    ModelMesh::SplitterFace,
    ModelMaterial::Mirror,
    [0.0, 0.0, 0.0],
)];

impl BlockRender for SplitterBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
