use super::*;
use crate::game::ui::components::{default_button_size, inset_border, HoverButton};
use crate::game::ui::types::{BlockMaterialIconSlot, LocalizedText};
use crate::game::ui::{BlockPanelDropdown, UiPanelId};
use crate::game::world::blocks::panel_layout::{panel_row_scene, spawn_block_panel};
use crate::game::world::blocks::ui_components::spawn_material_icon_dropdown_list;
use crate::game::world::blocks::{goal_settings, set_goal_settings, MaterialKind};
use crate::shared::i18n::I18n;
use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

pub(crate) fn spawn_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_block_panel(root, i18n, 430.0, "goal.title", UiPanelId::Goal, |panel| {
        panel
            .spawn_empty()
            .queue_apply_scene(panel_row_scene())
            .with_children(|row| {
                spawn_goal_label(row, "panel.material", i18n);
                spawn_goal_material_slot(
                    row,
                    BlockPanelDropdown::GoalMaterial,
                    BlockEditAction::ToggleMaterialDropdown,
                );
            });
    })
}

fn spawn_goal_label(row: &mut ChildSpawnerCommands, text_key: &'static str, i18n: &I18n) {
    row.spawn_empty()
        .queue_apply_scene(goal_label_scene(text_key, i18n));
}

fn goal_label_scene(text_key: &'static str, i18n: &I18n) -> impl bevy_scene::Scene {
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

fn spawn_goal_material_slot(
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
    .queue_apply_scene(goal_material_slot_visual_scene())
    .queue_spawn_related_scenes::<Children>(goal_material_icon_scene());
}

fn goal_material_slot_visual_scene() -> impl bevy_scene::Scene {
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

fn goal_material_icon_scene() -> impl bevy_scene::SceneList {
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
        BlockPanelDropdown::GoalMaterial,
        MaterialKind::ALL
            .into_iter()
            .map(|material| (material, BlockEditAction::SetMaterial(material))),
    )
}

pub(super) fn handle_edit_action(
    _block: &GoalBlock,
    ctx: &mut BlockEditContext,
    action: BlockEditAction,
) {
    let mut settings = goal_settings(ctx.world, ctx.pos);
    match action {
        BlockEditAction::ToggleMaterialDropdown => {
            ctx.toggle_dropdown(BlockPanelDropdown::GoalMaterial);
            return;
        }
        BlockEditAction::SetMaterial(material) => {
            settings.material = material;
            ctx.close_dropdown();
        }
        _ => return,
    }
    set_goal_settings(ctx.world, ctx.pos, settings);
    ctx.mark_dirty();
}
