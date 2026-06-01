use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &ReverseConveyorBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.reverse_conveyor",
        "short.reverse_conveyor",
        rgb(0.86, 0.46, 0.14),
        rgb(0.70, 0.34, 0.08),
    )
}
