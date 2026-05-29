use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, SceneBlock};

pub struct PlanksBlock;

pub static PLANKS: PlanksBlock = PlanksBlock;

impl Block for PlanksBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Planks
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::scene(
            self.id(),
            "block.planks",
            "short.planks",
            rgb(0.66, 0.45, 0.25),
            rgb(0.62, 0.40, 0.20),
        )
    }
}

impl SceneBlock for PlanksBlock {}
impl EditableBlock for PlanksBlock {}
