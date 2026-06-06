use super::{
    rgba, Block, BlockDefinition, BlockEditContext, BlockKind, EditableBlock, MaterialSource,
};
use crate::game::block_editing::{BlockPanelAction, BlockPanelDropdown};
use crate::game::state::UiPanelId;
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
}
impl EditableBlock for GeneratorBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Generator)
    }

    fn handle_edit_action(&self, ctx: &mut BlockEditContext, action: BlockPanelAction) {
        let mut settings = ctx.world.generator_settings(ctx.pos);
        match action {
            BlockPanelAction::PeriodDown => {
                settings.period = settings.period.saturating_sub(1).max(1)
            }
            BlockPanelAction::PeriodUp => settings.period = (settings.period + 1).min(120),
            BlockPanelAction::ToggleMaterialDropdown => {
                ctx.toggle_dropdown(BlockPanelDropdown::GeneratorMaterial);
                return;
            }
            BlockPanelAction::SetMaterial(material) => {
                settings.material = material;
                ctx.close_dropdown();
            }
            _ => return,
        }
        ctx.world.set_generator_settings(ctx.pos, settings);
        ctx.mark_dirty();
    }
}
