use super::{rgb, Block, BlockDefinition, BlockKind};

mod definition;

pub struct PlatformBlock;

pub static PLATFORM: PlatformBlock = PlatformBlock;

impl Block for PlatformBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Platform
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }
}
