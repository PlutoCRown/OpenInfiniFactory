use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &PusherBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.pusher",
        "short.pusher",
        rgb(0.54, 0.56, 0.54),
        rgb(0.42, 0.44, 0.42),
    )
}
