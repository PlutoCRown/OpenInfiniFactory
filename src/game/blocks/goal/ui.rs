use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::GoalBlock;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::widgets::{
    click_material_slot, spawn_material_icon_list, spawn_material_icon_toggle,
    sync_dropdown_overlay, update_material_icon,
};
use crate::game::block_editing::world_refresh::apply_block_settings_edit;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::{MaterialBlockId, material_catalog};
use crate::game::edit_history::EditHistory;
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::UiMainThread;
use crate::game::ui::components::{
    PanelOptions, default_button_size, localized_text, spawn_panel as spawn_ui_panel, text,
    transparent_node,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{CarriedItem, UiActionLabel, UiPanelBinding};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;

const MATERIAL_SLOT: u8 = 0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalAction {
    ToggleMaterial,
    SetMaterial(MaterialBlockId),
}

#[derive(Component, Clone, Copy)]
struct GoalMaterialSlot;

#[derive(Component, Clone, Copy)]
struct GoalMaterialList;

#[derive(Component, Clone, Copy)]
struct GoalMaterialOption(MaterialBlockId);

#[derive(Component, Clone, Copy)]
struct GoalAcceptorIdText;

impl UiActionLabel for GoalAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleMaterial | Self::SetMaterial(_) => "button.material_next",
        }
    }
}

impl BlockUi for GoalBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Goal)
    }
}

pub fn spawn_panel(root: &mut ChildSpawnerCommands) {
    spawn_ui_panel(
        root,
        PanelOptions::new(430.0, "goal.title").closable(),
        UiPanelBinding(UiPanelId::Goal),
        |panel| {
            spawn_row(panel, "panel.acceptor_id", |row| {
                row.spawn((text("-", 18.0, Color::WHITE), GoalAcceptorIdText));
            });
            spawn_row(panel, "panel.material", |row| {
                spawn_material_icon_toggle(row, GoalMaterialSlot, GoalAction::ToggleMaterial);
            });
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_list(
        root,
        GoalMaterialList,
        material_catalog()
            .iter()
            .map(|(id, _)| (id, GoalAction::SetMaterial(id))),
        GoalMaterialOption,
    );
}

pub fn register(app: &mut App) {
    app.add_observer(on_click).add_systems(
        Update,
        (update_panel, update_dropdowns)
            .chain()
            .in_set(BlockPanelSystems),
    );
}

inventory::submit! {
    BlockPanelHooks {
        panel: UiPanelId::Goal,
        spawn_panel: spawn_panel,
        spawn_overlays: spawn_overlays,
        register: register,
    }
}

fn spawn_row(
    panel: &mut ChildSpawnerCommands,
    label_key: &'static str,
    controls: impl FnOnce(&mut ChildSpawnerCommands),
) {
    panel
        .spawn(transparent_node(Node {
            width: Val::Percent(100.0),
            height: Val::Px(default_button_size(40.0)),
            display: Display::Flex,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        }))
        .with_children(|row| {
            row.spawn((
                localized_text(label_key, 16.0, Color::srgb(0.86, 0.88, 0.86)),
                Node {
                    width: Val::Px(110.0),
                    ..default()
                },
            ));
            controls(row);
        });
}

fn on_click(
    mut click: On<Pointer<Click>>,
    ui_host: Res<UiHost>,
    ui_runtime: Res<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut carried: ResMut<CarriedItem>,
    mut solution_state: ResMut<SolutionState>,
    mut edit_history: ResMut<EditHistory>,
    mut world: PlayingWorldParams,
    actions: Query<&GoalAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    if ui_runtime.active_panel() != Some(UiPanelId::Goal) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let mut settings = world.world.goal_settings(pos);

    let changed = match action {
        GoalAction::ToggleMaterial => {
            if let Some(material) = click_material_slot(
                UiPanelId::Goal,
                MATERIAL_SLOT,
                &mut carried,
                &mut open_dropdown,
            ) {
                settings.material = material;
                true
            } else {
                return;
            }
        }
        GoalAction::SetMaterial(material) => {
            settings.material = material;
            open_dropdown.close();
            true
        }
    };

    if changed {
        apply_block_settings_edit(&mut edit_history, &mut world, pos, |blocks| {
            blocks.set_goal_settings(pos, settings);
        });
        solution_state.dirty = true;
    }
}

fn update_panel(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut id_text: Query<&mut Text, With<GoalAcceptorIdText>>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    if ui_runtime.active_panel() != Some(UiPanelId::Goal) {
        return;
    }
    let label = world
        .acceptor_id_at(pos)
        .map(|id| format!("#{}", id.0))
        .unwrap_or_else(|| "-".to_string());
    for mut text in &mut id_text {
        if text.0 != label {
            text.0 = label.clone();
        }
    }
}

fn update_dropdowns(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    block_icons: Option<Res<BlockIconAssets>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut option_icons_filled: Local<bool>,
    mut last_slot_material: Local<Option<Option<MaterialBlockId>>>,
    mut material_slots: Query<(&GoalMaterialSlot, &Children)>,
    mut material_options: Query<(&GoalMaterialOption, &Children)>,
    mut material_icons: Query<&mut ImageNode>,
    mut lists: Query<(&GoalMaterialList, &mut Node, &ComputedNode)>,
    triggers: Query<(&GoalAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let panel = UiPanelId::Goal;
    let panel_active = ui_runtime.active_panel() == Some(panel);
    let open = panel_active && open_dropdown.is_open(panel, MATERIAL_SLOT);

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);
    for (_, mut style, list_node) in &mut lists {
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == GoalAction::ToggleMaterial && !node.is_empty()).then_some((node, transform))
        });
        sync_dropdown_overlay(open, &mut style, list_node, trigger, viewport);
    }

    let Some(icons) = block_icons.as_ref() else {
        return;
    };
    let icons_changed = icons.is_changed();
    let block_icons = icons.as_ref();
    if !*option_icons_filled || icons_changed {
        for (option, children) in &mut material_options {
            update_material_icon(children, Some(option.0), block_icons, &mut material_icons);
        }
        *option_icons_filled = true;
    }

    if !panel_active {
        *option_icons_filled = false;
        *last_slot_material = None;
        return;
    }
    let material = ui_runtime
        .active_block_pos()
        .map(|pos| world.goal_settings(pos).material);
    if last_slot_material.as_ref() != Some(&material) || icons_changed {
        for (_, children) in &mut material_slots {
            update_material_icon(children, material, block_icons, &mut material_icons);
        }
        *last_slot_material = Some(material);
    }
}
