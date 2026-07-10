use super::MirrorBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[BlockModelPart::new(
    ModelMesh::MirrorFace,
    ModelMaterial::Mirror,
    [0.0, 0.0, 0.0],
)
.yawed(std::f32::consts::FRAC_PI_2)];

impl BlockRender for MirrorBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
