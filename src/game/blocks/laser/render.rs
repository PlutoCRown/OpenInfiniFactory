use super::LaserBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh, RenderBehavior, WireConnectorBehavior};
use crate::game::world::direction::{Facing};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::Medium,
        ModelMaterial::DarkFrame,
        [0.0, 0.42, 0.08],
    ),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Laser, [0.0, 0.42, -0.30])
        .scaled([0.54, 0.54, 0.76]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Laser, [0.0, 0.42, -0.56]),
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Power, [0.0, 0.62, 0.08])
        .scaled([0.58, 0.58, 0.58]),
];

impl BlockRender for LaserBlock {
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        RenderBehavior {
            wire_connector: Some(WireConnectorBehavior::Device {
                blocked_offset: facing.forward_ivec3(),
            }),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
