use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock};

pub struct SolidBlock;

pub static SOLID: SolidBlock = SolidBlock;

impl Block for SolidBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Solid
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.solid",
            "short.solid",
            rgb(0.36, 0.47, 0.58),
            rgb(0.28, 0.38, 0.48),
        )
    }
}

impl FactoryBlock for SolidBlock {}
