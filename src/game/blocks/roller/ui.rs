use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::RollerBlock;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::widgets::{
    spawn_material_icon_list, spawn_material_icon_toggle, sync_dropdown_overlay, update_slot_icon,
};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::{PaintMaterialId, paint_catalog};
use crate::game::edit_history::{EditHistory, apply_block_settings_with_history};
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::{UiMainThread, i18n};
use crate::game::ui::components::{
    PanelOptions, default_button_size, localized_text, spawn_panel_with_title_marker,
    transparent_node,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{UiActionLabel, UiPanelBinding};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;

const COLOR_SLOT: u8 = 0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RollerAction {
    TogglePaint,
    SetPaint(PaintMaterialId),
}

#[derive(Component, Clone, Copy)]
pub struct RollerPanelTitle;

#[derive(Component, Clone, Copy)]
struct RollerPaintSlot;

#[derive(Component, Clone, Copy)]
struct RollerPaintList;

#[derive(Component, Clone, Copy)]
struct RollerPaintOption(PaintMaterialId);

impl UiActionLabel for RollerAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::TogglePaint | Self::SetPaint(_) => "button.next_color",
        }
    }
}

impl BlockUi for RollerBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Roller)
    }
}

pub fn spawn_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel_with_title_marker(
        root,
        PanelOptions::new(420.0, "roller.title").closable(),
        UiPanelBinding(UiPanelId::Roller),
        RollerPanelTitle,
        |panel| {
            spawn_row(panel, "panel.color", |row| {
                spawn_material_icon_toggle(row, RollerPaintSlot, RollerAction::TogglePaint);
            });
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_list(
        root,
        RollerPaintList,
        paint_catalog()
            .iter()
            .map(|(id, _)| (id, RollerAction::SetPaint(id))),
        RollerPaintOption,
    );
}

pub fn register(app: &mut App) {
    app.add_observer(on_click).add_systems(
        Update,
        (update_title, update_dropdowns)
            .chain()
            .in_set(BlockPanelSystems),
    );
}

inventory::submit! {
    BlockPanelHooks {
        panel: UiPanelId::Roller,
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
    mut edit_history: ResMut<EditHistory>,
    mut world: PlayingWorldParams,
    actions: Query<&RollerAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    if ui_runtime.active_panel() != Some(UiPanelId::Roller) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let mut settings = world.world.roller_settings(pos);
    let changed = match action {
        RollerAction::TogglePaint => {
            open_dropdown.toggle(UiPanelId::Roller, COLOR_SLOT);
            return;
        }
        RollerAction::SetPaint(paint) => {
            settings.paint = paint;
            open_dropdown.close();
            true
        }
    };

    if changed {
        apply_block_settings_with_history(edit_history.as_mut(), &mut world.world, pos, |blocks| {
            blocks.set_roller_settings(pos, settings);
        });
        solution_state.dirty = true;
        refresh_world_after_edit(&mut world, pos);
    }
}

fn update_title(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    mut titles: Query<&mut Text, With<RollerPanelTitle>>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Roller) {
        return;
    }
    let title = i18n.t("roller.title");
    for mut text in &mut titles {
        if text.0 != title {
            text.0 = title.clone();
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
    mut last_slot_paint: Local<Option<Option<PaintMaterialId>>>,
    mut paint_slots: Query<(&RollerPaintSlot, &Children)>,
    mut paint_options: Query<(&RollerPaintOption, &Children)>,
    mut icons: Query<&mut ImageNode>,
    mut lists: Query<(&RollerPaintList, &mut Node, &ComputedNode)>,
    triggers: Query<(&RollerAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let panel = UiPanelId::Roller;
    let panel_active = ui_runtime.active_panel() == Some(panel);
    let open = panel_active && open_dropdown.is_open(panel, COLOR_SLOT);

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);
    for (_, mut style, list_node) in &mut lists {
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == RollerAction::TogglePaint && !node.is_empty()).then_some((node, transform))
        });
        sync_dropdown_overlay(open, &mut style, list_node, trigger, viewport);
    }

    let Some(block_icons_res) = block_icons.as_ref() else {
        return;
    };
    let icons_changed = block_icons_res.is_changed();
    let block_icons = block_icons_res.as_ref();
    if !*option_icons_filled || icons_changed {
        for (option, children) in &mut paint_options {
            update_slot_icon(children, block_icons.paint(option.0), &mut icons);
        }
        *option_icons_filled = true;
    }

    if !panel_active {
        *last_slot_paint = None;
        return;
    }
    let paint = ui_runtime
        .active_block_pos()
        .map(|pos| world.roller_settings(pos).paint);
    if last_slot_paint.as_ref() != Some(&paint) || icons_changed {
        for (_, children) in &mut paint_slots {
            update_slot_icon(
                children,
                paint.and_then(|id| block_icons.paint(id)),
                &mut icons,
            );
        }
        *last_slot_paint = Some(paint);
    }
}
