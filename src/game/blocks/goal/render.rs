use super::GoalBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, BlockModelPart, ModelMaterial, ModelMesh, RenderBehavior};
use crate::game::world::direction::{Facing};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Goal, [0.0, 0.18, 0.0]),
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Goal, [0.0, 0.44, 0.0])
        .scaled([0.74, 0.74, 0.74]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Goal, [0.0, 0.66, 0.0]),
];

impl BlockRender for GoalBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            goal_topper: true,
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
