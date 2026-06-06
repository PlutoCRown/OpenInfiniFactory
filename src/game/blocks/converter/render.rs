use super::ConverterBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::System, [0.0, 0.38, 0.0]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::TeleportIn,
        [-0.28, 0.54, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::TeleportOut,
        [0.28, 0.54, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::SystemAccent,
        [0.0, 0.54, 0.0],
    )
    .scaled([0.62, 0.55, 0.55]),
];

impl BlockRender for ConverterBlock {
    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
