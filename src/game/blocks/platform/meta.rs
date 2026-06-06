use super::PlatformBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, BlockTexture, rgb, rgba};

impl BlockMeta for PlatformBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Platform
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.platform",
            "short.platform",
            rgb(0.36, 0.47, 0.58),
            rgb(0.28, 0.38, 0.48),
        )
        .textured(BlockTexture::Platform)
    }
}
