use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::session;
use crate::game::session::{puzzle_save_needs_confirm, save_current_world_resources};
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    StartMenuScreen, WorldEntryMode,
};
use crate::game::ui::access::{ui, UiMainThread};
use crate::game::ui::core::host::{PlayingUiRootEntity, UiHost, UiRootEntity};
use crate::game::ui::core::host::{UiAction, UiActionKind, UiInstanceId};
use crate::game::ui::core::runtime::UiPanelContext;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::save::open_save_as_new_puzzle_prompt;
use crate::game::ui::features::save::open_save_puzzle_confirm;
use crate::game::ui::types::{CarriedItem, InventoryItems};
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{next_solution_name_for_puzzle, SaveKind, SaveState};

use super::confirm::{
    on_reset_solution, on_return_to_main, on_save_before_edit, reset_solution_spec,
    return_to_main_spec, save_before_edit_spec,
};
use super::types::MenuAction;

pub fn emit_menu_actions(
    mut click: On<Pointer<Click>>,
    mut writer: MessageWriter<UiAction>,
    ui_host: Res<UiHost>,
    actions: Query<&MenuAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    writer.write(UiAction {
        instance: UiInstanceId::MENU,
        kind: UiActionKind::Menu(action),
    });
}

pub fn dispatch_menu_actions(
    _ui_thread: UiMainThread,
    mut actions: MessageReader<UiAction>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut world: ResMut<WorldBlocks>,
    mode: Res<State<GameMode>>,
    mut playing_ui: ResMut<PlayingUiState>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    ui_root: Option<Res<UiRootEntity>>,
    playing_ui_root: Option<Res<PlayingUiRootEntity>>,
    mut commands: Commands,
) {
    for action in actions.read() {
        if action.instance != UiInstanceId::MENU {
            continue;
        }
        let UiActionKind::Menu(action) = action.kind.clone() else {
            continue;
        };
        dispatch_menu_action(
            action,
            &mut builder_mode,
            &mut simulation,
            &mut inventory,
            &mut carried,
            &mut placement,
            &mut world,
            &mode,
            &mut playing_ui,
            &mut start_menu_screen,
            &mut save_state,
            &mut solution_state,
            ui_root.as_deref(),
            playing_ui_root.as_deref(),
            &mut commands,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_menu_action(
    action: MenuAction,
    builder_mode: &mut BuilderMode,
    simulation: &mut SimulationState,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    world: &mut WorldBlocks,
    mode: &State<GameMode>,
    playing_ui: &mut PlayingUiState,
    start_menu_screen: &mut StartMenuScreen,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    ui_root: Option<&UiRootEntity>,
    playing_ui_root: Option<&PlayingUiRootEntity>,
    commands: &mut Commands,
) {
    match (*mode.get(), action) {
        (GameMode::StartMenu, MenuAction::EditPuzzle) => {
            save_state.refresh();
            save_state.select_puzzle(None);
            solution_state.save_list_entry = WorldEntryMode::EditPuzzle;
            *start_menu_screen = StartMenuScreen::SaveList;
        }
        (GameMode::StartMenu, MenuAction::Play) => {
            save_state.refresh();
            save_state.select_puzzle(None);
            solution_state.save_list_entry = WorldEntryMode::PlaySolution;
            *start_menu_screen = StartMenuScreen::SaveList;
        }
        (GameMode::StartMenu, MenuAction::OpenSettings) => {
            ui.mount_settings(
                commands,
                ui_root.map(|root| root.0),
                UiPanelContext::SettingsFromStartMenu,
            );
        }
        (GameMode::StartMenu, MenuAction::Quit) => {
            std::process::exit(0);
        }
        (GameMode::Playing, MenuAction::Resume) => playing_ui.paused = false,
        (GameMode::Playing, MenuAction::ToggleBuilderMode) if playing_ui.paused => {
            if solution_state.entry == WorldEntryMode::PlaySolution {
                return;
            }
            *builder_mode = match *builder_mode {
                BuilderMode::Edit => {
                    simulation.running = false;
                    simulation.step_requested = false;
                    simulation.accumulator = 0.0;
                    simulation.start_snapshot = None;
                    simulation.start_structures = None;
                    solution_state.puzzle_snapshot = Some(world.clone());
                    solution_state.puzzle_id = save_state.current.clone();
                    save_state.current = Some(next_solution_name_for_puzzle(
                        &save_state
                            .entries
                            .iter()
                            .map(|entry| entry.name.clone())
                            .collect::<Vec<_>>(),
                        save_state.current.as_deref().unwrap_or("solution"),
                    ));
                    save_state.current_kind = Some(SaveKind::Solution);
                    BuilderMode::Play
                }
                BuilderMode::Play => {
                    ui.open_confirm_then(save_before_edit_spec(), on_save_before_edit);
                    return;
                }
            };
            *inventory = InventoryItems::for_mode(*builder_mode);
            carried.clear();
            placement.selected = 0;
            playing_ui.paused = false;
        }
        (GameMode::Playing, MenuAction::SaveWorld) if playing_ui.paused => {
            if puzzle_save_needs_confirm(save_state) {
                open_save_puzzle_confirm();
            } else {
                let _ = save_current_world_resources(
                    world,
                    inventory,
                    save_state,
                    solution_state,
                    simulation,
                );
            }
        }
        (GameMode::Playing, MenuAction::SaveAsNewPuzzle) if playing_ui.paused => {
            open_save_as_new_puzzle_prompt();
        }
        (GameMode::Playing, MenuAction::ResetSolution) if playing_ui.paused => {
            ui.open_confirm_then(reset_solution_spec(), on_reset_solution);
        }
        (GameMode::Playing, MenuAction::OpenSettings) if playing_ui.paused => {
            ui.mount_settings(
                commands,
                playing_ui_root.map(|root| root.0),
                UiPanelContext::SettingsFromPause,
            );
        }
        (GameMode::Playing, MenuAction::BackToMainMenu) if playing_ui.paused => {
            if solution_state.dirty {
                ui.open_confirm_then(return_to_main_spec(), on_return_to_main);
            } else {
                session::exit_to_main_menu(commands, false);
            }
        }
        _ => {}
    }
}
