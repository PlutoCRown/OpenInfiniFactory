use super::DetectorBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh, RenderBehavior, WireConnectorBehavior};
use crate::game::world::direction::{Facing};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Signal, [0.0, 0.52, 0.0]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Signal, [0.0, 0.38, -0.34])
        .scaled([0.72, 0.72, 0.55]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Power, [0.0, 0.38, -0.52]),
];

impl BlockRender for DetectorBlock {
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
