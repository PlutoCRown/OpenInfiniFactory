use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &RollerBlock) -> BlockDefinition {
    BlockDefinition::puzzle_system(
        block.id(),
        "block.roller",
        "short.roller",
        rgb(0.18, 0.62, 0.78),
        rgb(0.10, 0.44, 0.60),
    )
    .no_collision()
}
