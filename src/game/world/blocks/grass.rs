use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, SceneBlock};

pub struct GrassBlock;

pub static GRASS: GrassBlock = GrassBlock;

impl Block for GrassBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Grass
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::scene(
            self.id(),
            "block.grass",
            "short.grass",
            rgb(0.34, 0.62, 0.24),
            rgb(0.27, 0.56, 0.20),
        )
    }
}

impl SceneBlock for GrassBlock {}
impl EditableBlock for GrassBlock {}
