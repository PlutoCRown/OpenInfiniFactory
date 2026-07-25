use super::RotatorBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{
    BlockModel, BlockModelPart, ModelMaterial, ModelMesh, RenderBehavior, render_bottom_wire_device,
};
use crate::game::world::direction::Facing;

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
    BlockModelPart::new(ModelMesh::RotatorRing, ModelMaterial::Belt, [0.0, 0.48, 0.0]),
];

impl BlockRender for RotatorBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }

    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        render_bottom_wire_device()
    }
}
