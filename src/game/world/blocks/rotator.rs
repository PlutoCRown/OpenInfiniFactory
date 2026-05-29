use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock, MovementRule};

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
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: super::Facing) -> Option<MovementRule> {
        Some(MovementRule::Rotate { clockwise: true })
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::CounterRotator)
    }
}

impl FactoryBlock for RotatorBlock {}
