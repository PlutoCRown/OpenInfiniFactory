use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &SwitchBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.switch",
        "short.switch",
        rgb(0.90, 0.48, 0.18),
        rgb(0.78, 0.34, 0.12),
    )
}
