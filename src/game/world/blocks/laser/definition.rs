use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &LaserBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.laser",
        "short.laser",
        rgb(0.85, 0.20, 0.34),
        rgb(0.72, 0.12, 0.26),
    )
}
