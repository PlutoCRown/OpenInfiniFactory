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
        // 无玩家碰撞；模拟占用由 is_occupied（工厂恒占格）负责，下落不会覆盖
        .no_collision()
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Sign(SignSettings::default()))
    }
}
