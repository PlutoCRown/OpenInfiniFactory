use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &WelderBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.welder",
        "short.welder",
        rgb(0.14, 0.38, 0.74),
        rgb(0.08, 0.26, 0.58),
    )
}
