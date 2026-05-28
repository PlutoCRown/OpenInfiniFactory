use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock};

pub struct LifterBlock;

pub static LIFTER: LifterBlock = LifterBlock;

impl Block for LifterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Lifter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.lifter",
            "short.lifter",
            rgb(0.25, 0.58, 0.72),
            rgb(0.18, 0.48, 0.62),
        )
        .directional()
    }

    fn is_lifter(&self) -> bool {
        true
    }
}

impl FactoryBlock for LifterBlock {}
