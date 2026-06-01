use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &GeneratorBlock) -> BlockDefinition {
    BlockDefinition::puzzle_system(
        block.id(),
        "block.generator",
        "short.generator",
        rgba(0.42, 0.62, 1.0, 0.30),
        rgba(0.32, 0.48, 0.82, 0.46),
    )
    .no_collision()
    .transparent()
}
