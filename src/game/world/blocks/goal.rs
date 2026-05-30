use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, EditableBlock,
    ModelMaterial, ModelMesh, RenderBehavior, SystemBlock,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Goal, [0.0, 0.18, 0.0]),
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Goal, [0.0, 0.44, 0.0])
        .scaled([0.74, 0.74, 0.74]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Goal, [0.0, 0.66, 0.0]),
];

pub struct GoalBlock;

pub static GOAL: GoalBlock = GoalBlock;

impl Block for GoalBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Goal
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.goal",
            "short.goal",
            rgb(0.35, 0.72, 0.42),
            rgb(0.24, 0.56, 0.30),
        )
        .no_collision()
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        RenderBehavior {
            goal_topper: true,
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}

impl SystemBlock for GoalBlock {}
impl EditableBlock for GoalBlock {}
