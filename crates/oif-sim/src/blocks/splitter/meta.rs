use super::SplitterBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{rgb, BlockDefinition, BlockKind};

impl BlockMeta for SplitterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Splitter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.splitter",
            "short.splitter",
            "desc.splitter",
            rgb(0.45, 0.88, 1.0),
        )
        .transparent()
    }
}

