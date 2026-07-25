use super::SignBlock;

use glam::IVec3;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};
use crate::world::grid::{BlockSettings, SignSettings};

impl BlockMeta for SignBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Sign
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.sign",
            "short.sign",
            "desc.sign",
            rgb(0.72, 0.58, 0.32),
        )
        // 与短草同：不挡玩家；同格占用也按无碰撞处理
        .no_collision()
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Sign(SignSettings::default()))
    }
}
