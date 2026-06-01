use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &WeldPointBlock) -> BlockDefinition {
    BlockDefinition::marker(
        block.id(),
        "block.weld_point",
        "short.weld_point",
        rgba(1.0, 0.28, 0.18, 0.45),
        rgb(0.86, 0.16, 0.12),
    )
    .node()
    .transparent()
    .no_collision()
}
