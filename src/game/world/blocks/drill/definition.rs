use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &DrillBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.drill",
        "short.drill",
        rgb(0.32, 0.36, 0.40),
        rgb(0.24, 0.26, 0.30),
    )
}
