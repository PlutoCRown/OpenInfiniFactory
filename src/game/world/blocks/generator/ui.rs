use super::*;
use crate::game::ui::types::{BlockPanelText, BlockPanelTextKind};
use crate::game::world::blocks::panel_layout::{panel_text, spawn_block_panel, spawn_panel_row};
use crate::game::world::blocks::ui_components::{
    spawn_block_edit_button, spawn_material_icon_dropdown_list, spawn_material_icon_slot,
};
use crate::game::world::blocks::{generator_settings, set_generator_settings, MaterialKind};
use crate::shared::i18n::I18n;
use bevy::prelude::*;

pub(super) fn ui_panel(_block: &GeneratorBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Generator)
}

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_block_panel(
        root,
        i18n,
        430.0,
        "generator.title",
        UiPanelId::Generator,
        |panel| {
            spawn_panel_row(panel, i18n, "panel.period", |row| {
                spawn_block_edit_button(row, BlockEditAction::PeriodDown, "button.period_down");
                row.spawn((
                    panel_text("", 18.0, Color::WHITE),
                    BlockPanelText(BlockPanelTextKind::GeneratorPeriod),
                ));
                spawn_block_edit_button(row, BlockEditAction::PeriodUp, "button.period_up");
            });
            spawn_panel_row(panel, i18n, "panel.material", |row| {
                spawn_material_icon_slot(
                    row,
                    BlockPanelDropdown::GeneratorMaterial,
                    BlockEditAction::ToggleMaterialDropdown,
                );
            });
        },
    );
}

pub(crate) fn spawn_dropdown_layers(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::GeneratorMaterial,
        MaterialKind::ALL
            .into_iter()
            .map(|material| (material, BlockEditAction::SetMaterial(material))),
    );
}

pub(super) fn handle_edit_action(
    _block: &GeneratorBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    let mut settings = generator_settings(ctx.world, ctx.pos);
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
    set_generator_settings(ctx.world, ctx.pos, settings);
    ctx.mark_dirty();
}
