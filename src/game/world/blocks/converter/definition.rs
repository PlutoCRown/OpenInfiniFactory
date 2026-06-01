use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &ConverterBlock) -> BlockDefinition {
    BlockDefinition::puzzle_system(
        block.id(),
        "block.converter",
        "short.converter",
        rgb(0.50, 0.36, 0.78),
        rgb(0.36, 0.24, 0.62),
    )
    .no_collision()
}
