use super::VerticalMirrorBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[BlockModelPart::new(
    ModelMesh::VerticalMirrorFace,
    ModelMaterial::Mirror,
    [0.0, 0.0, 0.0],
)];

impl BlockRender for VerticalMirrorBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
