use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock, RotationDirection};

pub struct RotatorBlock;

pub static ROTATOR: RotatorBlock = RotatorBlock;

impl Block for RotatorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Rotator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.rotator",
            "short.rotator",
            rgb(0.48, 0.32, 0.72),
            rgb(0.42, 0.26, 0.64),
        )
        .directional()
        .alternate(BlockKind::CounterRotator)
    }

    fn rotation_direction(&self) -> Option<RotationDirection> {
        Some(RotationDirection::Clockwise)
    }
}

impl FactoryBlock for RotatorBlock {}
