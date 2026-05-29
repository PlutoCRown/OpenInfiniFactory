use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, SceneBlock};

pub struct DirtBlock;

pub static DIRT: DirtBlock = DirtBlock;

impl Block for DirtBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Dirt
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::scene(
            self.id(),
            "block.dirt",
            "short.dirt",
            rgb(0.40, 0.27, 0.16),
            rgb(0.42, 0.26, 0.14),
        )
    }
}

impl SceneBlock for DirtBlock {}
impl EditableBlock for DirtBlock {}
