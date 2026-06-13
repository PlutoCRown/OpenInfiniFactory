use super::DrillBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

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
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
