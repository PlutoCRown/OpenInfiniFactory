use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &GoalBlock) -> BlockDefinition {
    BlockDefinition::puzzle_system(
        block.id(),
        "block.goal",
        "short.goal",
        rgb(0.35, 0.72, 0.42),
        rgb(0.24, 0.56, 0.30),
    )
    .no_collision()
}
