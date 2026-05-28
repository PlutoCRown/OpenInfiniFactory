use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock};

pub struct WireBlock;

pub static WIRE: WireBlock = WireBlock;

impl Block for WireBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Wire
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.wire",
            "short.wire",
            rgb(0.95, 0.72, 0.18),
            rgb(0.88, 0.62, 0.12),
        )
        .node()
    }

    fn is_wire(&self) -> bool {
        true
    }
}

impl FactoryBlock for WireBlock {}
