use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    StartMenuScreen, WorldEntryMode,
};
use crate::game::ui::core::runtime::{UiPanelContext, UiRuntime};
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::core::world_menu::{
    return_to_main_menu, save_current_world, WorldMenuParams,
};
use crate::game::ui::features::save::types::{
    ConfirmDialogKind, ConfirmDialogState, TextPromptKind, TextPromptState,
};
use crate::game::ui::types::{CarriedItem, InventoryItems};
use crate::game::state::UiPanelId;
use crate::shared::save::{next_named_save, SaveKind, SaveState};

use super::types::MenuAction;

pub fn menu_actions(
    mut click: On<Pointer<Click>>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mode: Res<State<GameMode>>,
    mut playing_ui: ResMut<PlayingUiState>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut world_menu: WorldMenuParams,
    mut confirm_dialog: ResMut<ConfirmDialogState>,
    mut text_prompt: ResMut<TextPromptState>,
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
                    solution_state.puzzle_snapshot = Some(world_menu.world.clone());
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
                    confirm_dialog.kind = Some(ConfirmDialogKind::SaveSolutionBeforeEdit);
                    return;
                }
            };
            *inventory = InventoryItems::for_mode(*builder_mode);
            carried.clear();
            placement.selected = 0;
            playing_ui.paused = false;
        }
        (GameMode::Playing, MenuAction::SaveWorld) if playing_ui.paused => {
            save_current_world(
                &world_menu.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
                &simulation,
            );
        }
        (GameMode::Playing, MenuAction::SaveAsNewPuzzle) if playing_ui.paused => {
            crate::game::ui::core::world_menu::open_text_prompt(
                &mut text_prompt,
                TextPromptKind::SaveAsNewPuzzle,
                "puzzle",
            );
        }
        (GameMode::Playing, MenuAction::ResetSolution) if playing_ui.paused => {
            confirm_dialog.kind = Some(ConfirmDialogKind::ResetSolution);
        }
        (GameMode::Playing, MenuAction::OpenSettings) if playing_ui.paused => {
            ui_runtime.open(UiPanelId::Settings, UiPanelContext::SettingsFromPause);
        }
        (GameMode::Playing, MenuAction::BackToMainMenu) if playing_ui.paused => {
            if solution_state.dirty {
                confirm_dialog.kind = Some(ConfirmDialogKind::ReturnToMain);
            } else {
                return_to_main_menu(
                    &mut world_menu.world,
                    &mut placement,
                    &mut save_state,
                    &mut solution_state,
                    &mut simulation,
                    &mut world_menu.commands,
                    &mut world_menu.meshes,
                    world_menu.render_assets.as_deref(),
                    &world_menu.block_entities,
                    &world_menu.debug,
                    &mut world_menu.factory_structures,
                    &mut world_menu.movement_influence,
                    &mut world_menu.pusher_state,
                    &mut next_state,
                    &mut start_menu_screen,
                );
            }
        }
        _ => {}
    }
}
