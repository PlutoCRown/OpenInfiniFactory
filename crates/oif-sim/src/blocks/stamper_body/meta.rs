use super::StamperBodyBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for StamperBodyBlock {
    fn id(&self) -> BlockKind {
        BlockKind::StamperBody
    }

    fn definition(&self) -> BlockDefinition {
        // 有碰撞：写入 machine_bodies；朝向与宿主同步，对对齐印花透传
        BlockDefinition::marker(
            self.id(),
            "block.stamper_body",
            "short.stamper_body",
            "desc.stamper_body",
            rgb(0.72, 0.18, 0.48),
        )
    }
}
