use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock, RotationDirection};

pub struct CounterRotatorBlock;

pub static COUNTER_ROTATOR: CounterRotatorBlock = CounterRotatorBlock;

impl Block for CounterRotatorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::CounterRotator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.counter_rotator",
            "short.counter_rotator",
            rgb(0.62, 0.28, 0.78),
            rgb(0.54, 0.22, 0.68),
        )
        .directional()
        .alternate(BlockKind::Rotator)
    }

    fn rotation_direction(&self) -> Option<RotationDirection> {
        Some(RotationDirection::CounterClockwise)
    }
}

impl FactoryBlock for CounterRotatorBlock {}
