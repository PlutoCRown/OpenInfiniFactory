use super::*;
use crate::game::ui::types::ConverterInputRow;
use crate::game::world::blocks::panel_layout::{
    panel_row_node, spawn_block_panel, spawn_panel_label, spawn_panel_row,
};
use crate::game::world::blocks::ui_components::{
    spawn_material_icon_dropdown_list, spawn_material_icon_slot,
};
use crate::game::world::blocks::MaterialKind;
use crate::shared::i18n::I18n;
use bevy::prelude::*;

pub(super) fn ui_panel(_block: &ConverterBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Converter)
}

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_block_panel(
        root,
        i18n,
        460.0,
        "converter.title",
        UiPanelId::Converter,
        |panel| {
            panel
                .spawn((panel_row_node(), ConverterInputRow))
                .with_children(|row| {
                    spawn_panel_label(row, i18n, "panel.input");
                    spawn_material_icon_slot(
                        row,
                        BlockPanelDropdown::ConverterInput,
                        BlockEditAction::ToggleInputDropdown,
                    );
                });
            spawn_panel_row(panel, i18n, "panel.output", |row| {
                spawn_material_icon_slot(
                    row,
                    BlockPanelDropdown::ConverterOutput,
                    BlockEditAction::ToggleOutputDropdown,
                );
            });
        },
    );
}

pub(crate) fn spawn_dropdown_layers(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::ConverterInput,
        MaterialKind::ALL
            .into_iter()
            .map(|material| (material, BlockEditAction::SetInput(material))),
    );
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::ConverterOutput,
        MaterialKind::ALL
            .into_iter()
            .map(|material| (material, BlockEditAction::SetOutput(material))),
    );
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
