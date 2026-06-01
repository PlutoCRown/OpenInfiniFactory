use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &RotatorBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.rotator",
        "short.rotator",
        rgb(0.48, 0.32, 0.72),
        rgb(0.42, 0.26, 0.64),
    )
}
