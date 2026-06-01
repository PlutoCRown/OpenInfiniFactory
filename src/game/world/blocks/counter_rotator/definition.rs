use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &CounterRotatorBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.counter_rotator",
        "short.counter_rotator",
        rgb(0.62, 0.28, 0.78),
        rgb(0.54, 0.22, 0.68),
    )
}
