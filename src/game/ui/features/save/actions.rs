use bevy::input::keyboard::KeyboardInput;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::session::LoadWorld;
use crate::game::state::{GameMode, SolutionState, StartMenuScreen, WorldEntryMode};
use crate::game::ui::core::host::{UiAction, UiActionKind, UiHost, UiInstanceId};
use crate::game::ui::core::text_input::{primary_click, push_text_input};
use crate::game::ui::core::text_prompt::TextPromptState;
use crate::list_ui_config;
use crate::shared::save::SaveState;

use crate::game::ui::access::UiMainThread;

use super::confirm::open_delete_confirm;
use super::prompt::{
    open_new_puzzle_prompt, open_new_solution_prompt, open_rename_puzzle_prompt,
    open_rename_solution_prompt,
};
use super::types::SaveListAction;

struct SaveListCtx<'w> {
    start_menu_screen: &'w mut StartMenuScreen,
    save_state: &'w mut SaveState,
    solution_state: &'w SolutionState,
}

struct SaveListToolbarButton {
    action: SaveListAction,
    on_click: fn(&mut SaveListCtx<'_>),
}

const SAVE_LIST_TOOLBAR: &[SaveListToolbarButton] = list_ui_config!(
    SaveListToolbarButton,
    ctx: SaveListCtx<'_>,
    {
        for SaveListAction::NewPuzzle =>
        on_click(ctx) {
            if ctx.solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            open_new_puzzle_prompt();
        }
    };
    {
        for SaveListAction::NewSolution =>
        on_click(ctx) {
            if ctx.solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            let Some(puzzle_name) = ctx.save_state.selected_puzzle.clone() else {
                return;
            };
            open_new_solution_prompt(puzzle_name);
        }
    };
    {
        for SaveListAction::Back =>
        on_click(ctx) {
            *ctx.start_menu_screen = StartMenuScreen::Main;
        }
    }
);

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
        let mut ctx = SaveListCtx {
            start_menu_screen: &mut start_menu_screen,
            save_state: &mut save_state,
            solution_state: &solution_state,
        };
        if dispatch_save_list_toolbar(&action, &mut ctx) {
            continue;
        }
        dispatch_save_list_row_action(action, &mut ctx, &mut load_requests);
    }
}

fn dispatch_save_list_toolbar(action: &SaveListAction, ctx: &mut SaveListCtx<'_>) -> bool {
    for entry in SAVE_LIST_TOOLBAR {
        if &entry.action == action {
            (entry.on_click)(ctx);
            return true;
        }
    }
    false
}

fn dispatch_save_list_row_action(
    action: SaveListAction,
    ctx: &mut SaveListCtx<'_>,
    load_requests: &mut MessageWriter<LoadWorld>,
) {
    match action {
        SaveListAction::LoadPuzzle(name) => {
            if ctx.solution_state.save_list_entry == WorldEntryMode::EditPuzzle {
                if !ctx
                    .save_state
                    .puzzles()
                    .iter()
                    .any(|entry| entry.name == *name)
                {
                    return;
                }
                load_requests.write(LoadWorld {
                    name: name.clone(),
                    entry: WorldEntryMode::EditPuzzle,
                });
            } else if ctx
                .save_state
                .puzzles()
                .iter()
                .any(|entry| entry.name == *name)
            {
                ctx.save_state.select_puzzle(Some(name));
            }
        }
        SaveListAction::LoadSolution(name) => {
            if ctx.solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            if ctx.save_state.selected_puzzle.is_none() {
                return;
            }
            if !ctx
                .save_state
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
            if ctx.solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if ctx
                .save_state
                .puzzles()
                .iter()
                .any(|entry| entry.name == *name)
            {
                open_rename_puzzle_prompt(name.clone());
            }
        }
        SaveListAction::RenameSolution(name) => {
            if ctx.solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            if ctx
                .save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                open_rename_solution_prompt(name.clone());
            }
        }
        SaveListAction::DeletePuzzle(name) => {
            if ctx.solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if ctx
                .save_state
                .puzzles()
                .iter()
                .any(|entry| entry.name == *name)
            {
                open_delete_confirm(name.clone());
            }
        }
        SaveListAction::DeleteSolution(name) => {
            if ctx
                .save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                open_delete_confirm(name.clone());
            }
        }
        SaveListAction::NewPuzzle | SaveListAction::NewSolution | SaveListAction::Back => {}
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
