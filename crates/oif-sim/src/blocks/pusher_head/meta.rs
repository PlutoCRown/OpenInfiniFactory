use super::PusherHeadBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for PusherHeadBlock {
    fn id(&self) -> BlockKind {
        BlockKind::PusherHead
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.pusher_head",
            "short.pusher_head",
            "desc.pusher_head",
            rgb(0.45, 0.48, 0.52),
        )
        .node()
    }
}
