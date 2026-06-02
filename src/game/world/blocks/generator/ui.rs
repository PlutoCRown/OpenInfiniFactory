use super::*;
use crate::game::ui::components::{default_button_size, raised_border, HoverButton, BUTTON_BG};
use crate::game::ui::types::{BlockMaterialIconSlot, BlockPanelText, LocalizedText};
use crate::game::ui::UiPanelId;
use crate::game::world::blocks::panel_layout::{panel_row_scene, spawn_block_panel};
use crate::game::world::blocks::ui_components::spawn_material_icon_dropdown_list;
use crate::game::world::blocks::{generator_settings, set_generator_settings, MaterialKind};
use crate::shared::i18n::I18n;
use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_block_panel(
        root,
        i18n,
        430.0,
        "generator.title",
        UiPanelId::Generator,
        |panel| {
            panel
                .spawn_empty()
                .queue_apply_scene(panel_row_scene())
                .with_children(|row| {
                    spawn_generator_label(row, "panel.period", i18n);
                    spawn_generator_button(row, BlockEditAction::PeriodDown, "button.period_down");
                    row.spawn_empty()
                        .queue_spawn_related_scenes::<Children>(generator_period_text_scene());
                    spawn_generator_button(row, BlockEditAction::PeriodUp, "button.period_up");
                });
            panel
                .spawn_empty()
                .queue_apply_scene(panel_row_scene())
                .with_children(|row| {
                    spawn_generator_label(row, "panel.material", i18n);
                    spawn_generator_material_slot(
                        row,
                        BlockPanelDropdown::GeneratorMaterial,
                        BlockEditAction::ToggleMaterialDropdown,
                    );
                });
        },
    )
}

fn spawn_generator_label(row: &mut ChildSpawnerCommands, text_key: &'static str, i18n: &I18n) {
    row.spawn_empty()
        .queue_apply_scene(generator_label_scene(text_key, i18n));
}

fn generator_label_scene(text_key: &'static str, i18n: &I18n) -> impl bevy_scene::Scene {
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

fn spawn_generator_material_slot(
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
    .queue_apply_scene(generator_material_slot_visual_scene())
    .queue_spawn_related_scenes::<Children>(material_icon_scene());
}

fn generator_material_slot_visual_scene() -> impl bevy_scene::Scene {
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
            top: {crate::game::ui::components::inset_border().top},
            right: {crate::game::ui::components::inset_border().right},
            bottom: {crate::game::ui::components::inset_border().bottom},
            left: {crate::game::ui::components::inset_border().left},
        }
        BackgroundColor(Color::srgb(0.255, 0.251, 0.251))
    }
}

fn material_icon_scene() -> impl bevy_scene::SceneList {
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

fn spawn_generator_button(
    row: &mut ChildSpawnerCommands,
    action: BlockEditAction,
    text_key: &'static str,
) {
    row.spawn((Button, HoverButton, action))
        .queue_apply_scene(generator_button_visual_scene())
        .queue_spawn_related_scenes::<Children>(generator_button_label_scene(text_key));
}

fn generator_button_visual_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            height: Val::Px(default_button_size(36.0)),
            border: UiRect {
                left: Val::Px(3.0),
                right: Val::Px(3.0),
                top: Val::Px(4.0),
                bottom: Val::Px(5.0),
            },
            padding: UiRect::horizontal(Val::Px(14.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(BUTTON_BG)
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.62),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(4.0),
        )
    }
}

fn generator_button_label_scene(text_key: &'static str) -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text({text_key})
            TextFont {
                font_size: {crate::game::ui::components::default_font_size(14.0)}
            }
            TextColor(Color::WHITE)
            LocalizedText {
                key: {text_key}
            }
        )
    }
}

fn generator_period_text_scene() -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text("")
            TextFont {
                font_size: {crate::game::ui::components::default_font_size(18.0)}
            }
            TextColor(Color::WHITE)
            BlockPanelText {
                kind: {crate::game::ui::types::BlockPanelTextKind::GeneratorPeriod}
            }
        )
    }
}

pub(crate) fn spawn_dropdown_layers(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::GeneratorMaterial,
        MaterialKind::ALL
            .into_iter()
            .map(|material| (material, BlockEditAction::SetMaterial(material))),
    )
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
