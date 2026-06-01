use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &DownDetectorBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.down_detector",
        "short.down_detector",
        rgb(0.18, 0.52, 0.78),
        rgb(0.14, 0.40, 0.68),
    )
}
