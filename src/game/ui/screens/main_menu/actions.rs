use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{GameMode, SolutionState, WorldEntryMode};
use crate::game::systems::world_flow::primary_click;
use crate::game::ui::{MainMenuAction, UiPanelContext, UiPanelId, UiRuntime};
use crate::shared::save::SaveState;

pub fn main_menu_actions(
    mut click: On<Pointer<Click>>,
    mut mode: ResMut<GameMode>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut ui_runtime: ResMut<UiRuntime>,
    actions: Query<&MainMenuAction>,
) {
    if !primary_click(&mut click) || *mode != GameMode::MainMenu {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    match action {
        MainMenuAction::EditPuzzle => {
            save_state.refresh();
            save_state.select_puzzle(None, None);
            solution_state.save_list_entry = WorldEntryMode::EditPuzzle;
            *mode = GameMode::SaveListMain;
        }
        MainMenuAction::Play => {
            save_state.refresh();
            save_state.select_puzzle(None, None);
            solution_state.save_list_entry = WorldEntryMode::PlaySolution;
            *mode = GameMode::SaveListMain;
        }
        MainMenuAction::OpenSettings => {
            ui_runtime.open(
                UiPanelId::Settings,
                UiPanelContext::ReturnTo(GameMode::MainMenu),
            );
        }
        MainMenuAction::Quit => std::process::exit(0),
    }
}
