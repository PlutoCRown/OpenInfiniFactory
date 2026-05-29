use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, RenderBehavior, SystemBlock};

pub struct GoalBlock;

pub static GOAL: GoalBlock = GoalBlock;

impl Block for GoalBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Goal
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.goal",
            "short.goal",
            rgb(0.35, 0.72, 0.42),
            rgb(0.24, 0.56, 0.30),
        )
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        RenderBehavior {
            goal_topper: true,
            ..Default::default()
        }
    }
}

impl SystemBlock for GoalBlock {}
impl EditableBlock for GoalBlock {}
