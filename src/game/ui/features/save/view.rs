use bevy::prelude::*;

use crate::game::state::{GameMode, StartMenuScreen, WorldEntryMode};
use crate::game::ui::access::i18n;
use crate::shared::save::SaveState;

use super::types::{
    SaveListAction,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveListColumn {
    PuzzleEdit,
    PuzzlePlay,
    Solution,
}

impl SaveListColumn {
    pub fn create_action(self) -> Option<SaveListAction> {
        match self {
            Self::PuzzleEdit => Some(SaveListAction::NewPuzzle),
            Self::PuzzlePlay => None,
            Self::Solution => Some(SaveListAction::NewSolution),
        }
    }

    pub fn load(self, name: String) -> SaveListAction {
        match self {
            Self::PuzzleEdit | Self::PuzzlePlay => SaveListAction::LoadPuzzle(name),
            Self::Solution => SaveListAction::LoadSolution(name),
        }
    }

    pub fn rename(self, name: String) -> SaveListAction {
        match self {
            Self::PuzzleEdit => SaveListAction::RenamePuzzle(name),
            Self::PuzzlePlay => SaveListAction::LoadPuzzle(name),
            Self::Solution => SaveListAction::RenameSolution(name),
        }
    }

    pub fn delete(self, name: String) -> SaveListAction {
        match self {
            Self::PuzzleEdit => SaveListAction::DeletePuzzle(name),
            Self::PuzzlePlay => SaveListAction::LoadPuzzle(name),
            Self::Solution => SaveListAction::DeleteSolution(name),
        }
    }

    pub fn is_management(self) -> bool {
        !matches!(self, Self::PuzzlePlay)
    }
}

pub struct SaveListViewCtx<'a> {
    pub save_state: &'a SaveState,
    pub edit_flow: bool,
    pub play_flow: bool,
}

pub struct ActionButtonView {
    pub label: String,
    pub enabled: bool,
    pub selected: bool,
}

impl SaveListAction {
    pub fn button_view(&self, ctx: &SaveListViewCtx<'_>) -> ActionButtonView {
        ActionButtonView {
            label: self.button_label(ctx),
            enabled: self.button_enabled(ctx),
            selected: self.button_selected(ctx),
        }
    }

    fn button_label(&self, ctx: &SaveListViewCtx<'_>) -> String {
        let SaveListViewCtx {
            save_state,
            play_flow,
            ..
        } = ctx;
        match self {
            Self::LoadPuzzle(name) => {
                if *play_flow {
                    if save_state.selected_puzzle.as_deref() == Some(name.as_str()) {
                        i18n.fmt("save.selected_puzzle", &[("name", name.clone())])
                    } else {
                        i18n.fmt("save.select_puzzle", &[("name", name.clone())])
                    }
                } else {
                    i18n.fmt("save.load_puzzle", &[("name", name.clone())])
                }
            }
            Self::LoadSolution(name) => {
                i18n.fmt("save.load_solution", &[("name", name.clone())])
            }
            Self::RenamePuzzle(_) | Self::RenameSolution(_) => i18n.t("button.rename"),
            Self::DeletePuzzle(_) | Self::DeleteSolution(_) => i18n.t("button.delete"),
            Self::NewPuzzle => i18n.t("button.new_puzzle"),
            Self::NewSolution => i18n.t("button.new_solution"),
            Self::Back => i18n.t("button.back"),
        }
    }

    fn button_enabled(&self, ctx: &SaveListViewCtx<'_>) -> bool {
        let SaveListViewCtx {
            save_state,
            edit_flow,
            play_flow,
            ..
        } = ctx;
        match self {
            Self::LoadPuzzle(name) => {
                save_state
                    .puzzle_choices()
                    .iter()
                    .any(|choice| &choice.name == name)
                    || save_state.puzzles().iter().any(|entry| &entry.name == name)
            }
            Self::LoadSolution(name)
            | Self::RenameSolution(name)
            | Self::DeleteSolution(name) => {
                *play_flow
                    && save_state
                        .selected_puzzle_solutions()
                        .iter()
                        .any(|entry| &entry.name == name)
            }
            Self::RenamePuzzle(name) | Self::DeletePuzzle(name) => {
                *edit_flow && save_state.puzzles().iter().any(|entry| &entry.name == name)
            }
            Self::NewPuzzle => *edit_flow,
            Self::NewSolution => *play_flow && save_state.selected_puzzle.is_some(),
            Self::Back => true,
        }
    }

    fn button_selected(&self, ctx: &SaveListViewCtx<'_>) -> bool {
        matches!(
            self,
            Self::LoadPuzzle(name)
                if ctx.play_flow
                    && ctx.save_state.selected_puzzle.as_deref() == Some(name.as_str())
        )
    }
}


pub fn save_list_puzzle_rows(save_state: &SaveState, edit_flow: bool) -> Vec<String> {
    if edit_flow {
        save_state
            .puzzles()
            .into_iter()
            .map(|entry| entry.name.clone())
            .collect()
    } else {
        save_state
            .puzzle_choices()
            .into_iter()
            .map(|choice| choice.name)
            .collect()
    }
}

pub fn save_list_title(
    mode: GameMode,
    start_menu_screen: StartMenuScreen,
    entry: WorldEntryMode,
) -> String {
    match (mode, start_menu_screen) {
        (GameMode::StartMenu, StartMenuScreen::SaveList) => match entry {
            WorldEntryMode::EditPuzzle => i18n.t("save.title.edit_puzzle"),
            WorldEntryMode::PlaySolution => i18n.t("save.title.play_solution"),
        },
        _ => i18n.t("save.title.default"),
    }
}
