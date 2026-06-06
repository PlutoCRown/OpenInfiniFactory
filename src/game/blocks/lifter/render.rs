use super::LifterBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Lift, [0.0, 0.54, 0.0]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [-0.24, 0.18, -0.24]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [0.24, 0.18, -0.24]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [-0.24, 0.18, 0.24]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [0.24, 0.18, 0.24]),
];

impl BlockRender for LifterBlock {
    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
