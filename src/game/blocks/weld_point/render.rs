use super::WeldPointBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh, RenderBehavior, WeldConnectorBehavior};
use crate::game::world::direction::{Facing};

const MODEL: &[BlockModelPart] = &[BlockModelPart::new(
    ModelMesh::Small,
    ModelMaterial::WeldCore,
    [0.0, 0.0, 0.0],
)];

impl BlockRender for WeldPointBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::AllSides),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
