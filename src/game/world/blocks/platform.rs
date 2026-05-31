use super::{rgb, Block, BlockDefinition, BlockKind, BlockTexture};

pub struct PlatformBlock;

pub static PLATFORM: PlatformBlock = PlatformBlock;

impl Block for PlatformBlock {
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
