use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::session;
use crate::game::state::UiPanelId;
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    StartMenuScreen, WorldEntryMode,
};
use crate::game::ui::core::runtime::{UiPanelContext, UiRuntime};
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::save::{open_save_as_new_puzzle_prompt, SaveTextPromptParams};
use crate::game::ui::types::{CarriedItem, InventoryItems};
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{next_named_save, SaveKind, SaveState};

use super::confirm::{
    on_reset_solution, on_return_to_main, on_save_before_edit, reset_solution_spec,
    return_to_main_spec, save_before_edit_spec, MenuDialogParams,
};
use super::types::MenuAction;

pub fn menu_actions(
    mut click: On<Pointer<Click>>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    world: ResMut<WorldBlocks>,
    mode: Res<State<GameMode>>,
    mut playing_ui: ResMut<PlayingUiState>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut commands: Commands,
    mut dialog: MenuDialogParams,
    mut text_prompt: SaveTextPromptParams,
    actions: Query<&MenuAction>,
) {
    if !primary_click(&mut click) {
        return;
    }
    let Ok(action) = actions.get(click.entity).cloned() else {
        return;
    };
    click.propagate(false);

    match (*mode.get(), action) {
        (GameMode::StartMenu, MenuAction::EditPuzzle) => {
            save_state.refresh();
            save_state.select_puzzle(None, None);
            solution_state.save_list_entry = WorldEntryMode::EditPuzzle;
            *start_menu_screen = StartMenuScreen::SaveList;
        }
        (GameMode::StartMenu, MenuAction::Play) => {
            save_state.refresh();
            save_state.select_puzzle(None, None);
            solution_state.save_list_entry = WorldEntryMode::PlaySolution;
            *start_menu_screen = StartMenuScreen::SaveList;
        }
        (GameMode::StartMenu, MenuAction::OpenSettings) => {
            ui_runtime.open(UiPanelId::Settings, UiPanelContext::SettingsFromStartMenu);
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
                    simulation.start_factory_structures = None;
                    solution_state.puzzle_snapshot = Some(world.clone());
                    save_state.current = Some(next_named_save(
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
                    let spec = save_before_edit_spec(&dialog.i18n);
                    dialog.confirm.open_then(spec, on_save_before_edit);
                    return;
                }
            };
            *inventory = InventoryItems::for_mode(*builder_mode);
            carried.clear();
            placement.selected = 0;
            playing_ui.paused = false;
        }
        (GameMode::Playing, MenuAction::SaveWorld) if playing_ui.paused => {
            session::save_current_world(&mut commands);
        }
        (GameMode::Playing, MenuAction::SaveAsNewPuzzle) if playing_ui.paused => {
            open_save_as_new_puzzle_prompt(&mut text_prompt);
        }
        (GameMode::Playing, MenuAction::ResetSolution) if playing_ui.paused => {
            dialog
                .confirm
                .open_then(reset_solution_spec(&dialog.i18n), on_reset_solution);
        }
        (GameMode::Playing, MenuAction::OpenSettings) if playing_ui.paused => {
            ui_runtime.open(UiPanelId::Settings, UiPanelContext::SettingsFromPause);
        }
        (GameMode::Playing, MenuAction::BackToMainMenu) if playing_ui.paused => {
            if solution_state.dirty {
                dialog
                    .confirm
                    .open_then(return_to_main_spec(&dialog.i18n), on_return_to_main);
            } else {
                session::exit_to_main_menu(&mut commands, false);
            }
        }
        _ => {}
    }
}
