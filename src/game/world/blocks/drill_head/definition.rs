use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &DrillHeadBlock) -> BlockDefinition {
    BlockDefinition::marker(
        block.id(),
        "block.drill_head",
        "short.drill_head",
        rgb(0.12, 0.14, 0.16),
        rgb(0.10, 0.11, 0.12),
    )
    .node()
    .transparent()
    .no_collision()
}
