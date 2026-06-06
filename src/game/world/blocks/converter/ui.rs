use super::*;
use crate::game::ui::components::{default_button_size, inset_border, HoverButton};
use crate::game::ui::types::{BlockMaterialIconSlot, ConverterInputRow, LocalizedText};
use crate::game::ui::{BlockPanelDropdown, UiPanelId};
use crate::game::world::blocks::panel_layout::{panel_row_scene, spawn_block_panel};
use crate::game::world::blocks::ui_components::spawn_material_icon_dropdown_list;
use crate::game::world::blocks::{converter_settings, set_converter_settings, MaterialKind};
use crate::shared::i18n::I18n;
use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_block_panel(
        root,
        i18n,
        460.0,
        "converter.title",
        UiPanelId::Converter,
        |panel| {
            panel
                .spawn(ConverterInputRow)
                .queue_apply_scene(panel_row_scene())
                .with_children(|row| {
                    spawn_converter_label(row, "panel.input", i18n);
                    spawn_converter_material_slot(
                        row,
                        BlockPanelDropdown::ConverterInput,
                        BlockEditAction::ToggleInputDropdown,
                    );
                });
            panel
                .spawn(Visibility::Visible)
                .queue_apply_scene(panel_row_scene())
                .with_children(|row| {
                    spawn_converter_label(row, "panel.output", i18n);
                    spawn_converter_material_slot(
                        row,
                        BlockPanelDropdown::ConverterOutput,
                        BlockEditAction::ToggleOutputDropdown,
                    );
                });
        },
    )
}

fn spawn_converter_label(row: &mut ChildSpawnerCommands, text_key: &'static str, i18n: &I18n) {
    row.spawn(Visibility::Visible)
        .queue_apply_scene(converter_label_scene(text_key, i18n));
}

fn converter_label_scene(text_key: &'static str, i18n: &I18n) -> impl bevy_scene::Scene {
    let text = i18n.text(text_key);
    bsn! {
        Text({text})
        TextFont {
            font_size: {crate::game::ui::components::default_font_size(16.0)}
        }
        TextColor(Color::srgb(0.86, 0.88, 0.86))
        LocalizedText {
            key: {text_key}
        }
        Node {
            width: Val::Px(110.0),
        }
    }
}

fn spawn_converter_material_slot(
    row: &mut ChildSpawnerCommands,
    dropdown: BlockPanelDropdown,
    action: BlockEditAction,
) {
    row.spawn((
        Button,
        HoverButton,
        BlockMaterialIconSlot { dropdown },
        action,
    ))
    .queue_apply_scene(converter_material_slot_visual_scene())
    .queue_spawn_related_scenes::<Children>(converter_material_icon_scene());
}

fn converter_material_slot_visual_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Px(default_button_size(54.0)),
            height: Val::Px(default_button_size(54.0)),
            border: UiRect {
                left: Val::Px(4.0),
                right: Val::Px(4.0),
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BorderColor {
            top: {inset_border().top},
            right: {inset_border().right},
            bottom: {inset_border().bottom},
            left: {inset_border().left},
        }
        BackgroundColor(Color::srgb(0.255, 0.251, 0.251))
    }
}

fn converter_material_icon_scene() -> impl bevy_scene::SceneList {
    bsn! {
        (
            ImageNode::default()
            Node {
                width: Val::Px(default_button_size(64.0)),
                height: Val::Px(default_button_size(64.0)),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: {Val::Px(-default_button_size(32.0))},
                    top: {Val::Px(-default_button_size(32.0))},
                },
            }
        )
    }
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
    )
}

pub(super) fn handle_edit_action(
    _block: &ConverterBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    let mut settings = converter_settings(ctx.world, ctx.pos);
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
    set_converter_settings(ctx.world, ctx.pos, settings);
    ctx.mark_dirty();
}
