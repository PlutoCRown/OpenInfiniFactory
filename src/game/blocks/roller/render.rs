use super::RollerBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Large, ModelMaterial::System, [0.0, 0.38, 0.04]),
    BlockModelPart::new(ModelMesh::RodX, ModelMaterial::Signal, [0.0, 0.38, -0.40])
        .scaled([0.82, 0.82, 0.82]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::Signal,
        [-0.42, 0.38, -0.40],
    ),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Signal, [0.42, 0.38, -0.40]),
];

impl BlockRender for RollerBlock {
    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
