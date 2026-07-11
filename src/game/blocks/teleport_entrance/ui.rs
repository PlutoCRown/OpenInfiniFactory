use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::prompt::open_teleport_rename_prompt;
use super::TeleportEntranceBlock;

use crate::game::block_editing::widgets::{
    position_dropdown_from_trigger, spawn_labeled_panel_button, spawn_text_dropdown_toggle,
};
use crate::game::edit_history::{apply_teleport_pair_with_history, EditHistory};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::BlockKind;
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::{i18n, UiMainThread};
use crate::game::ui::components::{
    default_button_size, default_font_size, localized_text, menu_button,
    spawn_panel as spawn_ui_panel, text, transparent_node, PanelOptions,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{UiActionLabel, UiPanelBinding};
use crate::game::world::grid::WorldBlocks;

const PAIR_SLOT: u8 = 0;

#[derive(Resource, Default)]
struct PendingTeleportRename(Option<IVec3>);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeleportAction {
    StartRename,
    TogglePair,
    SetPair(Option<IVec3>),
}

#[derive(Component, Clone, Copy)]
struct TeleportNameText;

#[derive(Component, Clone, Copy)]
struct TeleportPairLabel;

#[derive(Component, Clone, Copy)]
struct TeleportPairList;

impl UiActionLabel for TeleportAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::StartRename => "button.teleport_rename",
            Self::TogglePair | Self::SetPair(_) => "button.teleport_pair",
        }
    }
}

impl BlockUi for TeleportEntranceBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Teleport)
    }
}

pub fn spawn_panel(root: &mut ChildSpawnerCommands) {
    spawn_ui_panel(
        root,
        PanelOptions::new(460.0, "teleport.title").closable(),
        UiPanelBinding(UiPanelId::Teleport),
        |panel| {
            spawn_row(panel, "panel.name", |row| {
                spawn_labeled_panel_button(row, TeleportAction::StartRename);
                row.spawn((text("", 18.0, Color::WHITE), TeleportNameText));
            });
            spawn_row(panel, "panel.pair", |row| {
                spawn_text_dropdown_toggle(row, TeleportAction::TogglePair, TeleportPairLabel);
            });
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Px(230.0),
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98)),
        GlobalZIndex(20_000),
        TeleportPairList,
    ));
}

pub fn register(app: &mut App) {
    app.init_resource::<PendingTeleportRename>()
        .add_observer(on_click)
        .add_systems(
            Update,
            (
                update_panel,
                update_dropdowns,
                process_teleport_rename_prompt,
            )
                .chain()
                .in_set(BlockPanelSystems),
        );
}

inventory::submit! {
    BlockPanelHooks {
        panel: UiPanelId::Teleport,
        spawn_panel: spawn_panel,
        spawn_overlays: spawn_overlays,
        register: register,
    }
}

pub fn dispatch_teleport_action(
    action: TeleportAction,
    pos: IVec3,
    _ui_runtime: &UiRuntime,
    world: &mut PlayingWorldParams,
    solution_state: &mut SolutionState,
    open_dropdown: &mut OpenBlockPanelDropdown,
    edit_history: &mut EditHistory,
) {
    match action {
        TeleportAction::StartRename => {
            debug_assert!(false, "rename is handled via text prompt");
        }
        TeleportAction::TogglePair => {
            open_dropdown.toggle(UiPanelId::Teleport, PAIR_SLOT);
        }
        TeleportAction::SetPair(pair) => {
            apply_teleport_pair_with_history(edit_history, &mut world.world, pos, pair);
            open_dropdown.close();
            solution_state.dirty = true;
            refresh_world_after_edit(world, pos);
        }
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
    mut pending_rename: ResMut<PendingTeleportRename>,
    mut solution_state: ResMut<SolutionState>,
    mut edit_history: ResMut<EditHistory>,
    mut world: PlayingWorldParams,
    actions: Query<&TeleportAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    if ui_runtime.active_panel() != Some(UiPanelId::Teleport) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    if action == TeleportAction::StartRename {
        pending_rename.0 = Some(pos);
        return;
    }
    dispatch_teleport_action(
        action,
        pos,
        &ui_runtime,
        &mut world,
        &mut solution_state,
        &mut open_dropdown,
        &mut edit_history,
    );
}

fn process_teleport_rename_prompt(
    _ui_thread: UiMainThread,
    mut pending_rename: ResMut<PendingTeleportRename>,
    world: Res<WorldBlocks>,
) {
    let Some(pos) = pending_rename.0.take() else {
        return;
    };
    if !world.system_blocks.contains_key(&pos) {
        return;
    }
    open_teleport_rename_prompt(pos, world.teleport_settings(pos).name);
}

fn update_panel(
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut name_text: Query<&mut Text, With<TeleportNameText>>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Teleport) {
        return;
    }
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    let settings = world.teleport_settings(pos);
    for mut text in &mut name_text {
        text.0 = settings.name.clone();
    }
}

#[derive(Component, Clone, Copy)]
struct TeleportPairOption;

fn update_dropdowns(
    _ui_thread: UiMainThread,
    mut commands: Commands,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut labels: Query<(&TeleportPairLabel, &mut Text)>,
    mut lists: Query<(
        Entity,
        &TeleportPairList,
        &mut Node,
        &ComputedNode,
        Option<&Children>,
    )>,
    pair_options: Query<Entity, With<TeleportPairOption>>,
    triggers: Query<(&TeleportAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
    mut pair_cache: Local<Option<(Option<IVec3>, u64, bool)>>,
) {
    let active_pos = ui_runtime.active_block_pos();
    let panel = UiPanelId::Teleport;

    if let Some(pos) = active_pos {
        let label = world
            .teleport_partner(pos)
            .map(|pair| world.teleport_settings(pair).name)
            .unwrap_or_else(|| i18n.t("teleport.none"));
        for (_, mut text) in &mut labels {
            text.0 = label.clone();
        }
    }

    let pair_open = open_dropdown.is_open(panel, PAIR_SLOT);
    let cache_key = (active_pos, world.topology_revision, pair_open);
    let rebuild = *pair_cache != Some(cache_key);
    if rebuild {
        *pair_cache = Some(cache_key);
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);

    for (entity, _, mut style, list_node, children) in &mut lists {
        style.display = if pair_open {
            Display::Flex
        } else {
            Display::None
        };
        if !pair_open {
            continue;
        }
        if rebuild {
            if let Some(children) = children {
                for child in children.iter() {
                    if pair_options.get(child).is_ok() {
                        commands.entity(child).despawn();
                    }
                }
            }
            if let Some(pos) = active_pos {
                commands.entity(entity).with_children(|parent| {
                    spawn_pair_option(parent, i18n.t("teleport.none"), None);
                    for candidate in pair_candidates(&world, pos) {
                        spawn_pair_option(
                            parent,
                            world.teleport_settings(candidate).name,
                            Some(candidate),
                        );
                    }
                });
            }
        }
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == TeleportAction::TogglePair && !node.is_empty()).then_some((node, transform))
        });
        if let Some((trigger_node, transform)) = trigger {
            if let Some((left, top)) =
                position_dropdown_from_trigger(trigger_node, transform, list_node, viewport)
            {
                style.left = Val::Px(left);
                style.top = Val::Px(top);
            }
        }
    }
}

fn spawn_pair_option(parent: &mut ChildSpawnerCommands, label: String, pair: Option<IVec3>) {
    parent
        .spawn((
            menu_button(32.0),
            TeleportAction::SetPair(pair),
            TeleportPairOption,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: default_font_size(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn pair_candidates(world: &WorldBlocks, pos: IVec3) -> Vec<IVec3> {
    let Some(block) = world.system_blocks.get(&pos) else {
        return Vec::new();
    };
    let target_kind = match block.kind {
        BlockKind::TeleportEntrance => BlockKind::TeleportExit,
        BlockKind::TeleportExit => BlockKind::TeleportEntrance,
        _ => return Vec::new(),
    };
    let mut candidates: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(candidate_pos, candidate)| {
            (candidate.kind == target_kind).then_some(*candidate_pos)
        })
        .collect();
    candidates.sort_by_key(|candidate| world.teleport_settings(*candidate).name);
    candidates
}
