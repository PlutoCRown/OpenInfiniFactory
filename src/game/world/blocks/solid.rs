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
            rgb(0.46, 0.48, 0.50),
            rgb(0.38, 0.39, 0.40),
        )
    }
}

impl FactoryBlock for SolidBlock {}
