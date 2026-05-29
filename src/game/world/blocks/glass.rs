use super::{rgb, rgba, Block, BlockDefinition, BlockKind, EditableBlock, SceneBlock};

pub struct GlassBlock;

pub static GLASS: GlassBlock = GlassBlock;

impl Block for GlassBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Glass
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::scene(
            self.id(),
            "block.glass",
            "short.glass",
            rgba(0.55, 0.82, 0.95, 0.45),
            rgb(0.42, 0.66, 0.76),
        )
        .transparent()
    }
}

impl SceneBlock for GlassBlock {}
impl EditableBlock for GlassBlock {}
