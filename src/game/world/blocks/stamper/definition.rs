use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &StamperBlock) -> BlockDefinition {
    BlockDefinition::puzzle_system(
        block.id(),
        "block.stamper",
        "short.stamper",
        rgb(0.82, 0.26, 0.58),
        rgb(0.64, 0.14, 0.42),
    )
    .no_collision()
}
