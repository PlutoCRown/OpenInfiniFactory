use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &WireBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.wire",
        "short.wire",
        rgb(0.95, 0.72, 0.18),
        rgb(0.88, 0.62, 0.12),
    )
    .node()
}
