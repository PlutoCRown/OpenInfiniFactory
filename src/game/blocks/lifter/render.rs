use super::LifterBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{
    BlockModel, BlockModelPart, ModelMaterial, ModelMesh, RenderBehavior, render_bottom_wire_device,
};
use crate::game::world::direction::Facing;

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

    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        render_bottom_wire_device()
    }
}
