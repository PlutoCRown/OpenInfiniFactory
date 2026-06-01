use super::*;

pub(super) fn default_settings(
    _block: &ConverterBlock,
    _pos: bevy::prelude::IVec3,
) -> Option<BlockSettings> {
    Some(BlockSettings::Converter(ConverterSettings::default()))
}

pub(super) fn ui_panel(_block: &ConverterBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Converter)
}

pub(super) fn handle_edit_action(
    _block: &ConverterBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    let mut settings = ctx.world.converter_settings(ctx.pos);
    match action {
        BlockEditAction::ToggleInputDropdown => {
            ctx.toggle_dropdown(BlockPanelDropdown::ConverterInput);
            return;
        }
        BlockEditAction::ToggleOutputDropdown => {
            ctx.toggle_dropdown(BlockPanelDropdown::ConverterOutput);
            return;
        }
        BlockEditAction::SetInput(material) => {
            settings.input = material;
            settings.mode = ConverterMode::SpecificInput;
            ctx.close_dropdown();
        }
        BlockEditAction::SetOutput(material) => {
            settings.output = material;
            ctx.close_dropdown();
        }
        _ => return,
    }
    ctx.world.set_converter_settings(ctx.pos, settings);
    ctx.mark_dirty();
}
