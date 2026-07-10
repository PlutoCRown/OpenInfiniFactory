mod confirm;

use bevy::prelude::*;

use crate::game::session;
use crate::game::session::{puzzle_save_needs_confirm, save_current_world_resources};
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    WorldEntryMode,
};
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::{i18n, ui, UiAccessScope, UiMainThread};
use crate::game::ui::core::host::PlayingUiRootEntity;
use crate::game::ui::core::runtime::UiPanelContext;
use crate::game::ui::features::save::{open_save_as_new_puzzle_prompt, open_save_puzzle_confirm};
use crate::game::ui::menu_button::{
    spawn_menu_button, MenuButtonClick, MenuButtonMarker, MenuButtonSet,
};
use crate::game::ui::types::{CarriedItem, InventoryItems};
use crate::game::world::grid::WorldBlocks;
use crate::list_ui_config;
use crate::shared::save::{next_named_save, SaveKind, SaveState};

use confirm::{
    on_reset_solution, on_return_to_main, on_save_before_edit, reset_solution_spec,
    return_to_main_spec, save_before_edit_spec,
};

pub struct PauseMenuPlugin;

struct PauseMenuCtx<'w> {
    builder_mode: &'w mut BuilderMode,
    simulation: &'w mut SimulationState,
    inventory: &'w mut InventoryItems,
    carried: &'w mut CarriedItem,
    placement: &'w mut PlacementState,
    world: &'w mut WorldBlocks,
    playing_ui: &'w mut PlayingUiState,
    save_state: &'w mut SaveState,
    solution_state: &'w mut SolutionState,
    playing_ui_root: Option<Entity>,
}

struct PauseMenuButton {
    label_key: &'static str,
    label: Option<fn(&SaveState) -> String>,
    visible: fn(&SaveState, &SolutionState) -> bool,
    on_click: fn(&mut PauseMenuCtx<'_>, &mut Commands),
}

const PAUSE_MENU_BUTTONS: &[PauseMenuButton] = list_ui_config!(
    PauseMenuButton,
    ctx: PauseMenuCtx<'_>,
    {
        key: "button.resume"
        on_click(ctx, _commands) {
            ctx.playing_ui.paused = false;
        }
    };
    {
        key: "button.toggle_builder_mode"
        visible(_save, solution) {
            solution.entry != WorldEntryMode::PlaySolution
        }
        on_click(ctx, _commands) {
            if ctx.solution_state.entry == WorldEntryMode::PlaySolution {
                return;
            }
            *ctx.builder_mode = match *ctx.builder_mode {
                BuilderMode::Edit => {
                    ctx.simulation.running = false;
                    ctx.simulation.step_requested = false;
                    ctx.simulation.accumulator = 0.0;
                    ctx.simulation.start_snapshot = None;
                    ctx.simulation.start_structures = None;
                    ctx.solution_state.puzzle_snapshot = Some(ctx.world.clone());
                    ctx.solution_state.puzzle_id = ctx.save_state.current.clone();
                    ctx.save_state.current = Some(next_named_save(
                        &ctx.save_state
                            .entries
                            .iter()
                            .map(|entry| entry.name.clone())
                            .collect::<Vec<_>>(),
                        ctx.save_state.current.as_deref().unwrap_or("solution"),
                    ));
                    ctx.save_state.current_kind = Some(SaveKind::Solution);
                    BuilderMode::Play
                }
                BuilderMode::Play => {
                    ui.open_confirm_then(save_before_edit_spec(), on_save_before_edit);
                    return;
                }
            };
            *ctx.inventory = InventoryItems::for_mode(*ctx.builder_mode);
            ctx.carried.clear();
            ctx.placement.selected = 0;
            ctx.playing_ui.paused = false;
        }
    };
    {
        key: "button.save_world"
        label(save) {
            match save.current_kind {
                Some(SaveKind::Solution) => i18n.t("button.save_solution"),
                _ => i18n.t("button.save_puzzle"),
            }
        }
        on_click(ctx, _commands) {
            if puzzle_save_needs_confirm(ctx.save_state) {
                open_save_puzzle_confirm();
            } else {
                let _ = save_current_world_resources(
                    ctx.world,
                    ctx.inventory,
                    ctx.save_state,
                    ctx.solution_state,
                    ctx.simulation,
                );
            }
        }
    };
    {
        key: "button.save_as_new_puzzle"
        visible(save, _solution) {
            save.current_kind == Some(SaveKind::Puzzle)
        }
        on_click(_ctx, _commands) {
            open_save_as_new_puzzle_prompt();
        }
    };
    {
        key: "button.reset_solution"
        visible(save, _solution) {
            save.current_kind == Some(SaveKind::Solution)
        }
        on_click(_ctx, _commands) {
            ui.open_confirm_then(reset_solution_spec(), on_reset_solution);
        }
    };
    {
        key: "button.settings"
        on_click(ctx, commands) {
            ui.mount_settings(
                commands,
                ctx.playing_ui_root,
                UiPanelContext::SettingsFromPause,
            );
        }
    };
    {
        key: "button.back_to_main_menu"
        on_click(ctx, commands) {
            if ctx.solution_state.dirty {
                ui.open_confirm_then(return_to_main_spec(), on_return_to_main);
            } else {
                session::exit_to_main_menu(commands, false);
            }
        }
    }
);

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                dispatch_pause_menu_clicks
                    .in_set(UiAccessScope)
                    .after(PerfScope::Input)
                    .before(PerfScope::Menus),
                sync_pause_menu_buttons
                    .in_set(UiAccessScope)
                    .after(crate::game::ui::update_localized_ui)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            ),
        );
    }
}

pub fn spawn_pause_menu_buttons(panel: &mut ChildSpawnerCommands) {
    for (index, button) in PAUSE_MENU_BUTTONS.iter().enumerate() {
        spawn_menu_button(
            panel,
            MenuButtonSet::PauseMenu,
            index as u8,
            button.label_key,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_pause_menu_clicks(
    _ui_thread: UiMainThread,
    mut clicks: MessageReader<MenuButtonClick>,
    mode: Res<State<GameMode>>,
    mut playing_ui: ResMut<PlayingUiState>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut world: ResMut<WorldBlocks>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    playing_ui_root: Option<Res<PlayingUiRootEntity>>,
    mut commands: Commands,
) {
    if *mode.get() != GameMode::Playing || !playing_ui.paused {
        return;
    }
    let playing_ui_root = playing_ui_root.as_deref().map(|root| root.0);
    for click in clicks.read() {
        if click.set != MenuButtonSet::PauseMenu {
            continue;
        }
        let Some(button) = PAUSE_MENU_BUTTONS.get(click.index as usize) else {
            continue;
        };
        let mut ctx = PauseMenuCtx {
            builder_mode: &mut builder_mode,
            simulation: &mut simulation,
            inventory: &mut inventory,
            carried: &mut carried,
            placement: &mut placement,
            world: &mut world,
            playing_ui: &mut playing_ui,
            save_state: &mut save_state,
            solution_state: &mut solution_state,
            playing_ui_root,
        };
        (button.on_click)(&mut ctx, &mut commands);
    }
}

fn sync_pause_menu_buttons(
    _ui_thread: UiMainThread,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    i18n_revision: Res<crate::game::ui::access::I18nRevision>,
    mut buttons: Query<(&MenuButtonMarker, &Children, &mut Node), With<Button>>,
    mut texts: Query<&mut Text>,
) {
    let labels_dirty = i18n_revision.is_changed() || save_state.is_changed();
    for (marker, children, mut node) in &mut buttons {
        if marker.set != MenuButtonSet::PauseMenu {
            continue;
        }
        let Some(button) = PAUSE_MENU_BUTTONS.get(marker.index as usize) else {
            continue;
        };
        let next = if (button.visible)(&save_state, &solution_state) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != next {
            node.display = next;
        }
        if !labels_dirty {
            continue;
        }
        let label = match button.label {
            Some(label) => label(&save_state),
            None => i18n.t(button.label_key),
        };
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}
