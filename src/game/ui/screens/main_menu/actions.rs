use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{GameMode, SolutionState, WorldEntryMode};
use crate::game::systems::world_flow::primary_click;
use crate::game::ui::{MainMenuAction, OpenUiPanel, SaveListChanged, UiPanelContext, UiPanelKey};
use crate::shared::save::SaveState;

pub fn main_menu_actions(
    mut click: On<Pointer<Click>>,
    actions: Query<&MainMenuAction>,
    mut mode: ResMut<GameMode>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut open_panel: MessageWriter<OpenUiPanel>,
    mut save_list_changed: MessageWriter<SaveListChanged>,
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
            save_list_changed.write(SaveListChanged);
            solution_state.save_list_entry = WorldEntryMode::EditPuzzle;
            *mode = GameMode::SaveListMain;
        }
        MainMenuAction::Play => {
            save_state.refresh();
            save_state.select_puzzle(None, None);
            save_list_changed.write(SaveListChanged);
            solution_state.save_list_entry = WorldEntryMode::PlaySolution;
            *mode = GameMode::SaveListMain;
        }
        MainMenuAction::OpenSettings => {
            open_panel.write(OpenUiPanel::new(
                UiPanelKey::SETTINGS,
                UiPanelContext::ReturnTo(GameMode::MainMenu),
            ));
        }
        MainMenuAction::Quit => std::process::exit(0),
    }
}
