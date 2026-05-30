use super::{rgba, Block, BlockDefinition, BlockKind, EditableBlock, MaterialSource, SystemBlock};
use crate::game::ui::UiPanelId;
use crate::game::world::grid::{BlockSettings, GeneratorSettings};

pub struct GeneratorBlock;

pub static GENERATOR: GeneratorBlock = GeneratorBlock;

impl Block for GeneratorBlock {
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

    fn material_source(&self, facing: super::Facing) -> Option<MaterialSource> {
        let _ = facing;
        Some(MaterialSource::Generator)
    }

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Generator(GeneratorSettings::default()))
    }

    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Generator)
    }
}

impl SystemBlock for GeneratorBlock {}
impl EditableBlock for GeneratorBlock {}
