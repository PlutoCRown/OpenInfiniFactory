use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, MaterialSource, SystemBlock};
use crate::game::world::grid::{BlockSettings, GeneratorSettings};

pub struct GeneratorBlock;

pub static GENERATOR: GeneratorBlock = GeneratorBlock;

impl Block for GeneratorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Generator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.generator",
            "short.generator",
            rgb(0.52, 0.30, 0.68),
            rgb(0.42, 0.20, 0.56),
        )
        .no_collision()
    }

    fn material_source(&self, facing: super::Facing) -> Option<MaterialSource> {
        let _ = facing;
        Some(MaterialSource::Generator)
    }

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Generator(GeneratorSettings::default()))
    }
}

impl SystemBlock for GeneratorBlock {}
impl EditableBlock for GeneratorBlock {}
