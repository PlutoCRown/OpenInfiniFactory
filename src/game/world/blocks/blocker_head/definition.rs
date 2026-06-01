use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &BlockerHeadBlock) -> BlockDefinition {
    BlockDefinition::marker(
        block.id(),
        "block.blocker_head",
        "short.blocker_head",
        rgb(0.54, 0.56, 0.54),
        rgb(0.42, 0.44, 0.42),
    )
}
