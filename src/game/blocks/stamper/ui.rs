use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::StamperBlock;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::widgets::{
    spawn_material_icon_list, spawn_material_icon_toggle, sync_dropdown_overlay, update_slot_icon,
};
use crate::game::block_editing::world_refresh::apply_block_settings_edit;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::{BlockKind, StampMaterialId, stamp_catalog};
use crate::game::edit_history::EditHistory;
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
pub enum StamperAction {
    ToggleStamp,
    SetStamp(StampMaterialId),
}

#[derive(Component, Clone, Copy)]
pub struct StamperPanelTitle;

#[derive(Component, Clone, Copy)]
struct StamperStampSlot;

#[derive(Component, Clone, Copy)]
struct StamperStampList;

#[derive(Component, Clone, Copy)]
struct StamperStampOption(StampMaterialId);

impl UiActionLabel for StamperAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleStamp | Self::SetStamp(_) => "button.next_color",
        }
    }
}

impl BlockUi for StamperBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Stamper)
    }
}

pub fn spawn_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel_with_title_marker(
        root,
        PanelOptions::new(420.0, "stamper.title").closable(),
        UiPanelBinding(UiPanelId::Stamper),
        StamperPanelTitle,
        |panel| {
            spawn_row(panel, "panel.color", |row| {
                spawn_material_icon_toggle(row, StamperStampSlot, StamperAction::ToggleStamp);
            });
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_list(
        root,
        StamperStampList,
        stamp_catalog()
            .iter()
            .map(|(id, _)| (id, StamperAction::SetStamp(id))),
        StamperStampOption,
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
        panel: UiPanelId::Stamper,
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
    actions: Query<&StamperAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    if ui_runtime.active_panel() != Some(UiPanelId::Stamper) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let mut settings = world.world.stamper_settings(pos);
    let changed = match action {
        StamperAction::ToggleStamp => {
            open_dropdown.toggle(UiPanelId::Stamper, COLOR_SLOT);
            return;
        }
        StamperAction::SetStamp(stamp) => {
            settings.stamp = stamp;
            open_dropdown.close();
            true
        }
    };

    if changed {
        apply_block_settings_edit(edit_history.as_mut(), &mut world, pos, |blocks| {
            blocks.set_stamper_settings(pos, settings);
        });
        solution_state.dirty = true;
    }
}

fn update_title(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    mut titles: Query<&mut Text, With<StamperPanelTitle>>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Stamper) {
        return;
    }
    let title = i18n.t("stamper.title");
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
    mut last_slot_stamp: Local<Option<Option<StampMaterialId>>>,
    mut stamp_slots: Query<(&StamperStampSlot, &Children)>,
    mut stamp_options: Query<(&StamperStampOption, &Children)>,
    mut icons: Query<&mut ImageNode>,
    mut lists: Query<(&StamperStampList, &mut Node, &ComputedNode)>,
    triggers: Query<(&StamperAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let panel = UiPanelId::Stamper;
    let panel_active = ui_runtime.active_panel() == Some(panel);
    let open = panel_active && open_dropdown.is_open(panel, COLOR_SLOT);

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);
    for (_, mut style, list_node) in &mut lists {
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == StamperAction::ToggleStamp && !node.is_empty()).then_some((node, transform))
        });
        sync_dropdown_overlay(open, &mut style, list_node, trigger, viewport);
    }

    let Some(block_icons_res) = block_icons.as_ref() else {
        return;
    };
    let icons_changed = block_icons_res.is_changed();
    let block_icons = block_icons_res.as_ref();
    if !*option_icons_filled || icons_changed {
        for (option, children) in &mut stamp_options {
            update_slot_icon(
                children,
                block_icons.get(BlockKind::stamp_block_kind(option.0)),
                &mut icons,
            );
        }
        *option_icons_filled = true;
    }

    if !panel_active {
        *option_icons_filled = false;
        *last_slot_stamp = None;
        return;
    }
    let stamp = ui_runtime
        .active_block_pos()
        .map(|pos| world.stamper_settings(pos).stamp);
    if last_slot_stamp.as_ref() != Some(&stamp) || icons_changed {
        for (_, children) in &mut stamp_slots {
            update_slot_icon(
                children,
                stamp.and_then(|id| block_icons.get(BlockKind::stamp_block_kind(id))),
                &mut icons,
            );
        }
        *last_slot_stamp = Some(stamp);
    }
}
