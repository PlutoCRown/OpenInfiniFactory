use super::CounterRotatorBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::RotatorBase,
        ModelMaterial::PlatformBase,
        [0.0, 0.0, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RotatorDisk,
        ModelMaterial::ConveyorBelt,
        [0.0, 0.50, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RotatorRing,
        ModelMaterial::Belt,
        [0.0, 0.50, 0.0],
    ),
];

impl BlockRender for CounterRotatorBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
