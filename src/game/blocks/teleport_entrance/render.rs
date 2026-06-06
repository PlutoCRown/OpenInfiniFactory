use super::TeleportEntranceBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::TeleportIn,
        [0.0, 0.22, -0.30],
    )
    .scaled([0.88, 0.72, 0.72]),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::TeleportIn,
        [0.0, 0.22, 0.30],
    )
    .scaled([0.88, 0.72, 0.72]),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::TeleportIn,
        [-0.30, 0.22, 0.0],
    )
    .scaled([0.72, 0.72, 0.88]),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::TeleportIn,
        [0.30, 0.22, 0.0],
    )
    .scaled([0.72, 0.72, 0.88]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::TeleportIn,
        [0.0, 0.42, 0.0],
    )
    .scaled([0.88, 0.88, 0.88]),
];

impl BlockRender for TeleportEntranceBlock {
    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
