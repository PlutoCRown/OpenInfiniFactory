use bevy::input::keyboard::KeyboardInput;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::session;
use crate::game::state::{GameMode, SolutionState, StartMenuScreen, WorldEntryMode};
use crate::game::ui::core::text_input::{primary_click, push_text_input};
use crate::game::ui::core::text_prompt::TextPromptState;
use crate::shared::save::SaveState;

use super::confirm::{open_delete_confirm, SaveDialogParams};
use super::prompt::{
    open_new_puzzle_prompt, open_new_solution_prompt, open_rename_puzzle_prompt,
    open_rename_solution_prompt, SaveTextPromptParams,
};
use super::types::SaveListAction;

pub fn save_list_actions(
    mut click: On<Pointer<Click>>,
    mode: Res<State<GameMode>>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut save_state: ResMut<SaveState>,
    solution_state: Res<SolutionState>,
    mut commands: Commands,
    mut confirm: SaveDialogParams,
    mut text_prompt: SaveTextPromptParams,
    actions: Query<&SaveListAction>,
) {
    if !primary_click(&mut click)
        || *mode.get() != GameMode::StartMenu
        || *start_menu_screen != StartMenuScreen::SaveList
    {
        return;
    }
    let Ok(action) = actions.get(click.entity).cloned() else {
        return;
    };
    click.propagate(false);

    match action {
        SaveListAction::NewPuzzle => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            open_new_puzzle_prompt(&mut text_prompt);
        }
        SaveListAction::NewSolution => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            let Some(puzzle_name) = save_state.selected_puzzle.clone() else {
                return;
            };
            open_new_solution_prompt(&mut text_prompt, puzzle_name);
        }
        SaveListAction::LoadPuzzle(name) => {
            if solution_state.save_list_entry == WorldEntryMode::EditPuzzle {
                if !save_state.puzzles().iter().any(|entry| entry.name == *name) {
                    return;
                }
                session::load_world(&mut commands, name.clone(), WorldEntryMode::EditPuzzle);
            } else {
                let Some(choice) = save_state
                    .puzzle_choices()
                    .into_iter()
                    .find(|choice| choice.name == *name)
                else {
                    return;
                };
                save_state.select_puzzle(Some(choice.name), Some(choice.source));
            }
        }
        SaveListAction::LoadSolution(name) => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            if save_state.selected_puzzle.is_none() {
                return;
            }
            if !save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                return;
            }
            session::load_world(&mut commands, name.clone(), WorldEntryMode::PlaySolution);
        }
        SaveListAction::RenamePuzzle(name) => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if save_state.puzzles().iter().any(|entry| entry.name == *name) {
                open_rename_puzzle_prompt(&mut text_prompt, name.clone());
            }
        }
        SaveListAction::RenameSolution(name) => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            if save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                open_rename_solution_prompt(&mut text_prompt, name.clone());
            }
        }
        SaveListAction::DeletePuzzle(name) => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if save_state.puzzles().iter().any(|entry| entry.name == *name) {
                open_delete_confirm(&mut confirm, name.clone());
            }
        }
        SaveListAction::DeleteSolution(name) => {
            if save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                open_delete_confirm(&mut confirm, name.clone());
            }
        }
        SaveListAction::Back => {
            *start_menu_screen = StartMenuScreen::Main;
        }
    }
}

pub fn text_prompt_input(
    mut prompt: ResMut<TextPromptState>,
    mut keyboard_input: MessageReader<KeyboardInput>,
) {
    if !prompt.is_open() {
        return;
    }
    let mut submit = false;
    let mut cancel = false;
    for event in keyboard_input.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            bevy::input::keyboard::Key::Enter => submit = true,
            bevy::input::keyboard::Key::Escape => cancel = true,
            bevy::input::keyboard::Key::Backspace => {
                prompt.value.pop();
            }
            _ => push_text_input(&mut prompt.value, event),
        }
    }
    if submit {
        prompt.submit();
    } else if cancel {
        prompt.cancel();
    }
}
