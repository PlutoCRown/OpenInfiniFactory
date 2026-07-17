use super::SignBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

// 薄板立于格内靠后，朝向为文字面
const MODEL: &[BlockModelPart] = &[BlockModelPart::new(
    ModelMesh::Plate,
    ModelMaterial::Wood,
    [0.0, 0.05, 0.38],
)];

impl BlockRender for SignBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
