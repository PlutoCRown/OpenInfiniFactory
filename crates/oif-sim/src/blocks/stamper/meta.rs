use super::StamperBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};
use glam::IVec3;
use crate::world::grid::{BlockSettings, StamperSettings};

impl BlockMeta for StamperBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Stamper
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.stamper",
            "short.stamper",
            "desc.stamper",
            rgb(0.82, 0.26, 0.58),
        )
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Stamper(StamperSettings::default()))
    }
}
