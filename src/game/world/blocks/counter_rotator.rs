use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock, MovementRule};

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
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: super::Facing) -> Option<MovementRule> {
        Some(MovementRule::Rotate { clockwise: false })
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Rotator)
    }
}

impl FactoryBlock for CounterRotatorBlock {}
