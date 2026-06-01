use super::*;
use crate::game::world::blocks::*;

pub(super) fn definition(block: &PlatformBlock) -> BlockDefinition {
    BlockDefinition::factory(
        block.id(),
        "block.platform",
        "short.platform",
        rgb(0.36, 0.47, 0.58),
        rgb(0.28, 0.38, 0.48),
    )
    .textured(BlockTexture::Platform)
}
