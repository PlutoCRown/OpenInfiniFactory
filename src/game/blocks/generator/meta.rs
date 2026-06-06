use super::GeneratorBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgba};
use bevy::prelude::{IVec3};
use crate::game::world::grid::{BlockSettings, GeneratorSettings};

impl BlockMeta for GeneratorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Generator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.generator",
            "short.generator",
            rgba(0.42, 0.62, 1.0, 0.30),
            rgba(0.32, 0.48, 0.82, 0.46),
        )
        .no_collision()
        .transparent()
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Generator(GeneratorSettings::default()))
    }
}
