use super::RollerBodyBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for RollerBodyBlock {
    fn id(&self) -> BlockKind {
        BlockKind::RollerBody
    }

    fn definition(&self) -> BlockDefinition {
        // 有碰撞：写入 machine_bodies，阻挡材料进入；无模型由游戏侧 PartsOnly 空表处理
        BlockDefinition::marker(
            self.id(),
            "block.roller_body",
            "short.roller_body",
            "desc.roller_body",
            rgb(0.55, 0.22, 0.12),
        )
    }
}
