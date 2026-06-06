use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::GoalBlock;

use crate::game::block_editing::widgets::{
    position_dropdown_from_trigger, spawn_labeled_panel_button, spawn_material_icon_list,
    spawn_material_icon_toggle, ui_transform_scale, update_material_icon,
};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::block_editing::{BlockEditContext, OpenBlockPanelDropdown};
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::MaterialKind;
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::UiMainThread;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::components::{
    default_button_size, localized_text, spawn_panel as spawn_ui_panel, text, transparent_node,
    PanelOptions,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::types::{UiActionLabel, UiPanelBinding};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;

const MATERIAL_SLOT: u8 = 0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalAction {
    ToggleMaterial,
    SetMaterial(MaterialKind),
}

#[derive(Component, Clone, Copy)]
struct GoalMaterialSlot;

#[derive(Component, Clone, Copy)]
struct GoalMaterialList;

#[derive(Component, Clone, Copy)]
struct GoalMaterialOption(MaterialKind);

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
        MaterialKind::ALL
            .into_iter()
            .map(|material| (material, GoalAction::SetMaterial(material))),
        GoalMaterialOption,
    );
}

pub fn register(app: &mut App) {
    app.add_observer(on_click)
        .add_systems(Update, update_dropdowns.in_set(BlockPanelSystems));
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
    mut solution_state: ResMut<SolutionState>,
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

    let mut ctx = BlockEditContext::new(
        pos,
        &mut world.world,
        &mut solution_state,
        &mut open_dropdown,
    );
    let mut settings = ctx.world.goal_settings(pos);
    let mut changed = false;

    match action {
        GoalAction::ToggleMaterial => {
            ctx.toggle_dropdown(UiPanelId::Goal, MATERIAL_SLOT);
            return;
        }
        GoalAction::SetMaterial(material) => {
            settings.material = material;
            ctx.close_dropdown();
            changed = true;
        }
    }

    if changed {
        ctx.world.set_goal_settings(pos, settings);
        ctx.mark_dirty();
        refresh_world_after_edit(&mut world);
    }
}

fn update_dropdowns(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    block_icons: Option<Res<BlockIconAssets>>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut material_slots: Query<(&GoalMaterialSlot, &Children)>,
    mut material_options: Query<(&GoalMaterialOption, &Children)>,
    mut material_icons: Query<&mut ImageNode>,
    mut lists: Query<(&GoalMaterialList, &mut Node, &ComputedNode)>,
    triggers: Query<(&GoalAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let active_pos = ui_runtime.active_block_pos();
    let panel = UiPanelId::Goal;

    if let Some(block_icons) = block_icons.as_deref() {
        let material = active_pos.map(|pos| world.goal_settings(pos).material);
        for (_, children) in &mut material_slots {
            update_material_icon(children, material, block_icons, &mut material_icons);
        }
        for (option, children) in &mut material_options {
            update_material_icon(children, Some(option.0), block_icons, &mut material_icons);
        }
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);
    let scale = ui_transform_scale(window, ui_scale.0);

    for (_, mut style, list_node) in &mut lists {
        let open = open_dropdown.is_open(panel, MATERIAL_SLOT);
        style.display = if open { Display::Flex } else { Display::None };
        if !open {
            continue;
        }
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == GoalAction::ToggleMaterial && !node.is_empty()).then_some((node, transform))
        });
        if let Some((trigger_node, transform)) = trigger {
            if let Some((left, top)) = position_dropdown_from_trigger(
                trigger_node,
                transform,
                list_node.size(),
                viewport,
                scale,
            ) {
                style.left = Val::Px(left);
                style.top = Val::Px(top);
            }
        }
    }
}
