use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &DetectorBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.detector",
        "short.detector",
        rgb(0.15, 0.45, 0.72),
        rgb(0.12, 0.34, 0.62),
    )
}
