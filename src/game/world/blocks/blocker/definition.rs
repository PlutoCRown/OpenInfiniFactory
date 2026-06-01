use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &BlockerBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.blocker",
        "short.blocker",
        rgb(0.54, 0.56, 0.54),
        rgb(0.42, 0.44, 0.42),
    )
}
