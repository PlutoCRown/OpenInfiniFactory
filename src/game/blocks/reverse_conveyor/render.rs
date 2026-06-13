use super::ReverseConveyorBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::ConveyorBase,
        ModelMaterial::Belt,
        [0.0, 0.0, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::ConveyorBelt,
        ModelMaterial::ConveyorBelt,
        [0.0, -0.46, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::BeltStripe,
        [-0.11, -0.52, 0.26],
    )
    .scaled([0.62, 0.16, 0.42])
    .yawed(-0.7853982),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::BeltStripe,
        [0.11, -0.52, 0.26],
    )
    .scaled([0.62, 0.16, 0.42])
    .yawed(0.7853982),
];

impl BlockRender for ReverseConveyorBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
