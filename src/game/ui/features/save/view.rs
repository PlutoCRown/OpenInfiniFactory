use bevy::prelude::*;

use crate::game::ui::access::i18n;
use crate::shared::save::SaveState;

use super::types::SaveListAction;

pub struct SaveListViewCtx<'a> {
    pub save_state: &'a SaveState,
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
            enabled: self.is_enabled(ctx.save_state),
            selected: self.button_selected(ctx),
        }
    }

    /// 是否可点（不碰 i18n，observer 里也能用）
    pub fn is_enabled(&self, save_state: &SaveState) -> bool {
        match self {
            Self::SelectPuzzle(storage) => save_state
                .puzzles()
                .iter()
                .any(|entry| entry.slot.puzzle == *storage),
            Self::SelectSolution(storage) => save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.slot.solution.as_deref() == Some(storage.as_str())),
            Self::NewPuzzle | Self::Back => true,
            Self::NewSolution => save_state.selected_puzzle.is_some(),
            Self::EditSelectedPuzzle | Self::RenameSelectedPuzzle | Self::DeleteSelectedPuzzle => {
                save_state.selected_puzzle.is_some()
            }
            Self::RenameSelectedSolution | Self::DeleteSelectedSolution | Self::StartGame => {
                save_state.selected_solution.is_some()
            }
        }
    }

    fn button_label(&self, ctx: &SaveListViewCtx<'_>) -> String {
        let save_state = ctx.save_state;
        match self {
            Self::SelectPuzzle(storage) => puzzle_display_name(save_state, storage),
            Self::SelectSolution(storage) => solution_display_name(save_state, storage),
            Self::NewPuzzle => i18n.t("button.new_puzzle"),
            Self::NewSolution => i18n.t("button.new_solution"),
            Self::EditSelectedPuzzle => i18n.t("button.edit_puzzle"),
            Self::RenameSelectedPuzzle | Self::RenameSelectedSolution => String::new(),
            Self::DeleteSelectedPuzzle | Self::DeleteSelectedSolution => String::new(),
            Self::StartGame => i18n.t("button.start_game"),
            Self::Back => i18n.t("button.back"),
        }
    }

    fn button_selected(&self, ctx: &SaveListViewCtx<'_>) -> bool {
        match self {
            Self::SelectPuzzle(storage) => {
                ctx.save_state.selected_puzzle.as_deref() == Some(storage.as_str())
            }
            Self::SelectSolution(storage) => {
                ctx.save_state.selected_solution.as_deref() == Some(storage.as_str())
            }
            _ => false,
        }
    }
}

fn puzzle_display_name(save_state: &SaveState, storage: &str) -> String {
    save_state
        .puzzles()
        .iter()
        .find(|entry| entry.slot.puzzle == storage)
        .map(|entry| entry.name.clone())
        .unwrap_or_else(|| storage.to_string())
}

fn solution_display_name(save_state: &SaveState, storage: &str) -> String {
    save_state
        .selected_puzzle_solutions()
        .iter()
        .find(|entry| entry.slot.solution.as_deref() == Some(storage))
        .map(|entry| entry.name.clone())
        .unwrap_or_else(|| storage.to_string())
}

pub fn save_list_puzzle_rows(save_state: &SaveState) -> Vec<String> {
    save_state
        .puzzles()
        .into_iter()
        .map(|entry| entry.slot.puzzle.clone())
        .collect()
}

pub fn save_list_title() -> String {
    i18n.t("save.title.play_solution")
}
