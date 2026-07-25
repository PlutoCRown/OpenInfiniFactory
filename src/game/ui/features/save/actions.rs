use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::session::{LoadWorld, SessionBusy, SessionBusyCover};
use crate::game::state::{GameMode, StartMenuScreen, WorldEntryMode};
use crate::game::ui::access::UiMainThread;
use crate::game::ui::core::host::{UiAction, UiActionKind, UiHost, UiInstanceId};
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::types::{SaveListCoverImage, SaveListRenderState};
use crate::shared::save::{SaveSlot, SaveState};

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
    save_state: Res<SaveState>,
    ui_host: Res<UiHost>,
    busy: Res<SessionBusy>,
    mut writer: MessageWriter<UiAction>,
    actions: Query<&SaveListAction>,
) {
    if busy.is_busy()
        || ui_host.modal_open()
        || !primary_click(&mut click)
        || *mode.get() != GameMode::StartMenu
        || *start_menu_screen != StartMenuScreen::SaveList
    {
        return;
    }
    let Ok(action) = actions.get(click.entity).cloned() else {
        return;
    };
    if !action.is_enabled(&save_state) {
        return;
    }
    click.propagate(false);
    writer.write(UiAction {
        instance: UiInstanceId::SAVE_LIST,
        kind: UiActionKind::SaveList(action.clone()),
    });
    // 方案卡双击：选中后直接开始游戏
    if click.event.count >= 2 {
        if let SaveListAction::SelectSolution(storage) = &action {
            if save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.slot.solution.as_deref() == Some(storage.as_str()))
            {
                writer.write(UiAction {
                    instance: UiInstanceId::SAVE_LIST,
                    kind: UiActionKind::SaveList(SaveListAction::StartGame),
                });
            }
        }
    }
}

pub fn dispatch_save_list_actions(
    _ui_thread: UiMainThread,
    mut actions: MessageReader<UiAction>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut save_state: ResMut<SaveState>,
    mut busy_cover: ResMut<SessionBusyCover>,
    busy: Res<SessionBusy>,
    render_state: Res<SaveListRenderState>,
    mut load_requests: MessageWriter<LoadWorld>,
    cover_images: Query<(&ImageNode, &Node), With<SaveListCoverImage>>,
) {
    if busy.is_busy() {
        return;
    }
    for action in actions.read() {
        if action.instance != UiInstanceId::SAVE_LIST {
            continue;
        }
        let UiActionKind::SaveList(action) = action.kind.clone() else {
            continue;
        };
        match action {
            SaveListAction::NewPuzzle => open_new_puzzle_prompt(),
            SaveListAction::NewSolution => {
                let Some(puzzle_name) = save_state.selected_puzzle.clone() else {
                    continue;
                };
                open_new_solution_prompt(puzzle_name);
            }
            SaveListAction::Back => {
                *start_menu_screen = StartMenuScreen::Main;
            }
            SaveListAction::SelectPuzzle(storage) => {
                if save_state
                    .puzzles()
                    .iter()
                    .any(|entry| entry.slot.puzzle == storage)
                {
                    save_state.select_puzzle(Some(storage));
                }
            }
            SaveListAction::SelectSolution(storage) => {
                if save_state
                    .selected_puzzle_solutions()
                    .iter()
                    .any(|entry| entry.slot.solution.as_deref() == Some(storage.as_str()))
                {
                    save_state.select_solution(Some(storage));
                }
            }
            SaveListAction::EditSelectedPuzzle => {
                let Some(puzzle) = save_state.selected_puzzle.clone() else {
                    continue;
                };
                if !save_state
                    .puzzles()
                    .iter()
                    .any(|entry| entry.slot.puzzle == puzzle)
                {
                    continue;
                }
                capture_busy_cover(&cover_images, &render_state, &save_state, &mut busy_cover);
                load_requests.write(LoadWorld {
                    slot: SaveSlot::puzzle(puzzle),
                    entry: WorldEntryMode::EditPuzzle,
                });
            }
            SaveListAction::RenameSelectedPuzzle => {
                let Some(puzzle) = save_state.selected_puzzle.clone() else {
                    continue;
                };
                let Some(entry) = save_state
                    .puzzles()
                    .iter()
                    .find(|entry| entry.slot.puzzle == puzzle)
                    .map(|entry| (*entry).clone())
                else {
                    continue;
                };
                open_rename_puzzle_prompt(entry.slot, entry.name);
            }
            SaveListAction::DeleteSelectedPuzzle => {
                let Some(puzzle) = save_state.selected_puzzle.clone() else {
                    continue;
                };
                if save_state
                    .puzzles()
                    .iter()
                    .any(|entry| entry.slot.puzzle == puzzle)
                {
                    open_delete_confirm(SaveSlot::puzzle(puzzle));
                }
            }
            SaveListAction::RenameSelectedSolution => {
                let Some(puzzle) = save_state.selected_puzzle.clone() else {
                    continue;
                };
                let Some(solution) = save_state.selected_solution.clone() else {
                    continue;
                };
                let Some(entry) = save_state
                    .selected_puzzle_solutions()
                    .iter()
                    .find(|entry| entry.slot.solution.as_deref() == Some(solution.as_str()))
                    .cloned()
                else {
                    continue;
                };
                if entry.slot.puzzle != puzzle {
                    continue;
                }
                open_rename_solution_prompt(entry.slot, entry.name);
            }
            SaveListAction::DeleteSelectedSolution => {
                let Some(puzzle) = save_state.selected_puzzle.clone() else {
                    continue;
                };
                let Some(solution) = save_state.selected_solution.clone() else {
                    continue;
                };
                if save_state
                    .selected_puzzle_solutions()
                    .iter()
                    .any(|entry| entry.slot.solution.as_deref() == Some(solution.as_str()))
                {
                    open_delete_confirm(SaveSlot::solution(puzzle, solution));
                }
            }
            SaveListAction::StartGame => {
                let Some(puzzle) = save_state.selected_puzzle.clone() else {
                    continue;
                };
                let Some(solution) = save_state.selected_solution.clone() else {
                    continue;
                };
                if !save_state
                    .selected_puzzle_solutions()
                    .iter()
                    .any(|entry| entry.slot.solution.as_deref() == Some(solution.as_str()))
                {
                    continue;
                }
                capture_busy_cover(&cover_images, &render_state, &save_state, &mut busy_cover);
                load_requests.write(LoadWorld {
                    slot: SaveSlot::solution(puzzle, solution),
                    entry: WorldEntryMode::PlaySolution,
                });
            }
        }
    }
}

/// 把存档列表当前封面接到加载遮罩（仅当封面已对上当前选中）
fn capture_busy_cover(
    cover_images: &Query<(&ImageNode, &Node), With<SaveListCoverImage>>,
    render_state: &SaveListRenderState,
    save_state: &SaveState,
    busy_cover: &mut SessionBusyCover,
) {
    let expected = if let Some(solution) = save_state.selected_solution.as_ref() {
        save_state
            .selected_puzzle
            .as_ref()
            .map(|puzzle| SaveSlot::solution(puzzle.clone(), solution.clone()).storage_path())
    } else {
        save_state
            .selected_puzzle
            .as_ref()
            .map(|puzzle| SaveSlot::puzzle(puzzle.clone()).storage_path())
    };
    let Some(expected) = expected else {
        busy_cover.clear();
        return;
    };
    if render_state.cover_key.as_deref() != Some(expected.as_str()) {
        busy_cover.clear();
        return;
    }
    busy_cover.image = cover_images
        .iter()
        .find(|(_, node)| node.display != Display::None)
        .map(|(image, _)| image.image.clone());
}
