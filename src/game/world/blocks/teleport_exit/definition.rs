use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &TeleportExitBlock) -> BlockDefinition {
    BlockDefinition::puzzle_system(
        block.id(),
        "block.teleport_exit",
        "short.teleport_exit",
        rgb(0.72, 0.34, 0.96),
        rgb(0.50, 0.20, 0.74),
    )
    .no_collision()
}
