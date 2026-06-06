use super::StamperBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Large, ModelMaterial::System, [0.0, 0.38, 0.04]),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::SystemAccent,
        [0.0, 0.38, -0.30],
    )
    .scaled([0.56, 0.56, 0.58]),
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Laser, [0.0, 0.38, -0.54])
        .scaled([0.52, 0.70, 0.40]),
];

impl BlockRender for StamperBlock {
    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
