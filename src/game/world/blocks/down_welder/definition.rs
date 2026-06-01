use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &DownWelderBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.down_welder",
        "short.down_welder",
        rgb(0.14, 0.38, 0.74),
        rgb(0.08, 0.26, 0.58),
    )
}
