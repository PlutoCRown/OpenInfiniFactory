use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &ConveyorBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.conveyor",
        "short.conveyor",
        rgb(0.86, 0.46, 0.14),
        rgb(0.70, 0.34, 0.08),
    )
}
