use super::*;

pub(super) fn default_settings(
    _block: &GeneratorBlock,
    _pos: bevy::prelude::IVec3,
) -> Option<BlockSettings> {
    Some(BlockSettings::Generator(GeneratorSettings::default()))
}

pub(super) fn ui_panel(_block: &GeneratorBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Generator)
}

pub(super) fn handle_edit_action(
    _block: &GeneratorBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    let mut settings = ctx.world.generator_settings(ctx.pos);
    match action {
        BlockEditAction::PeriodDown => settings.period = settings.period.saturating_sub(1).max(1),
        BlockEditAction::PeriodUp => settings.period = (settings.period + 1).min(120),
        BlockEditAction::ToggleMaterialDropdown => {
            ctx.toggle_dropdown(BlockPanelDropdown::GeneratorMaterial);
            return;
        }
        BlockEditAction::SetMaterial(material) => {
            settings.material = material;
            ctx.close_dropdown();
        }
        _ => return,
    }
    ctx.world.set_generator_settings(ctx.pos, settings);
    ctx.mark_dirty();
}
