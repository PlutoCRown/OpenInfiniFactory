use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, SceneBlock};

pub struct StoneBlock;

pub static STONE: StoneBlock = StoneBlock;

impl Block for StoneBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Stone
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::scene(
            self.id(),
            "block.stone",
            "short.stone",
            rgb(0.43, 0.43, 0.42),
            rgb(0.42, 0.42, 0.40),
        )
    }
}

impl SceneBlock for StoneBlock {}
impl EditableBlock for StoneBlock {}
