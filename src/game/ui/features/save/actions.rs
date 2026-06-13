use bevy::input::keyboard::KeyboardInput;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::session::LoadWorld;
use crate::game::state::{GameMode, SolutionState, StartMenuScreen, WorldEntryMode};
use crate::game::ui::core::host::{UiAction, UiActionKind, UiHost, UiInstanceId};
use crate::game::ui::core::text_input::{primary_click, push_text_input};
use crate::game::ui::core::text_prompt::TextPromptState;
use crate::shared::save::SaveState;

use crate::game::ui::access::UiMainThread;

use super::confirm::open_delete_confirm;
use super::prompt::{
    open_new_puzzle_prompt, open_new_solution_prompt, open_rename_puzzle_prompt,
    open_rename_solution_prompt,
};
use super::types::SaveListAction;

pub fn emit_save_list_actions(
    mut click: On<Pointer<Click>>,
    mode: Res<State<GameMode>>,
    start_menu_screen: Res<StartMenuScreen>,
    ui_host: Res<UiHost>,
    mut writer: MessageWriter<UiAction>,
    actions: Query<&SaveListAction>,
) {
    if ui_host.modal_open()
        || !primary_click(&mut click)
        || *mode.get() != GameMode::StartMenu
        || *start_menu_screen != StartMenuScreen::SaveList
    {
        return;
    }
    let Ok(action) = actions.get(click.entity).cloned() else {
        return;
    };
    click.propagate(false);
    writer.write(UiAction {
        instance: UiInstanceId::SAVE_LIST,
        kind: UiActionKind::SaveList(action),
    });
}

pub fn dispatch_save_list_actions(
    _ui_thread: UiMainThread,
    mut actions: MessageReader<UiAction>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut save_state: ResMut<SaveState>,
    solution_state: Res<SolutionState>,
    mut load_requests: MessageWriter<LoadWorld>,
) {
    for action in actions.read() {
        if action.instance != UiInstanceId::SAVE_LIST {
            continue;
        }
        let UiActionKind::SaveList(action) = action.kind.clone() else {
            continue;
        };
        dispatch_save_list_action(
            action,
            &mut start_menu_screen,
            &mut save_state,
            &solution_state,
            &mut load_requests,
        );
    }
}

fn dispatch_save_list_action(
    action: SaveListAction,
    start_menu_screen: &mut StartMenuScreen,
    save_state: &mut SaveState,
    solution_state: &SolutionState,
    load_requests: &mut MessageWriter<LoadWorld>,
) {
    match action {
        SaveListAction::NewPuzzle => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            open_new_puzzle_prompt();
        }
        SaveListAction::NewSolution => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            let Some(puzzle_name) = save_state.selected_puzzle.clone() else {
                return;
            };
            open_new_solution_prompt(puzzle_name);
        }
        SaveListAction::LoadPuzzle(name) => {
            if solution_state.save_list_entry == WorldEntryMode::EditPuzzle {
                if !save_state.puzzles().iter().any(|entry| entry.name == *name) {
                    return;
                }
                load_requests.write(LoadWorld {
                    name: name.clone(),
                    entry: WorldEntryMode::EditPuzzle,
                });
            } else if save_state.puzzles().iter().any(|entry| entry.name == *name) {
                save_state.select_puzzle(Some(name));
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
            load_requests.write(LoadWorld {
                name: name.clone(),
                entry: WorldEntryMode::PlaySolution,
            });
        }
        SaveListAction::RenamePuzzle(name) => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if save_state.puzzles().iter().any(|entry| entry.name == *name) {
                open_rename_puzzle_prompt(name.clone());
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
                open_rename_solution_prompt(name.clone());
            }
        }
        SaveListAction::DeletePuzzle(name) => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if save_state.puzzles().iter().any(|entry| entry.name == *name) {
                open_delete_confirm(name.clone());
            }
        }
        SaveListAction::DeleteSolution(name) => {
            if save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                open_delete_confirm(name.clone());
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
    host: Res<UiHost>,
    mut actions: MessageWriter<UiAction>,
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
    let Some(instance) = host.active_text_prompt_instance() else {
        return;
    };
    if submit {
        actions.write(UiAction {
            instance,
            kind: UiActionKind::TextPromptSubmit {
                value: prompt.value.clone(),
            },
        });
    } else if cancel {
        actions.write(UiAction {
            instance,
            kind: UiActionKind::TextPromptCancel,
        });
    }
}
