use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &LifterBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.lifter",
        "short.lifter",
        rgb(0.25, 0.58, 0.72),
        rgb(0.18, 0.48, 0.62),
    )
}
