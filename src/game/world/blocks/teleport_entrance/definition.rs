use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &TeleportEntranceBlock) -> BlockDefinition {
    BlockDefinition::puzzle_system(
        block.id(),
        "block.teleport_entrance",
        "short.teleport_entrance",
        rgb(0.12, 0.62, 0.92),
        rgb(0.06, 0.42, 0.72),
    )
    .no_collision()
}
