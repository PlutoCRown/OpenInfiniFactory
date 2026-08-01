use bevy::prelude::*;

use crate::game::ui::access::i18n;
use crate::shared::save::{SaveKind, SaveState};

use super::types::SaveListAction;

pub struct SaveListViewCtx<'a> {
    pub save_state: &'a SaveState,
}

pub struct ActionButtonView {
    pub label: String,
    pub enabled: bool,
    pub selected: bool,
    /// 非 None 时强制显隐（Free 选中时隐藏方案/编辑谜题相关按钮）
    pub display: Option<Display>,
}

impl SaveListAction {
    pub fn button_view(&self, ctx: &SaveListViewCtx<'_>) -> ActionButtonView {
        let free_selected = selected_is_free(ctx.save_state);
        ActionButtonView {
            label: self.button_label(ctx),
            enabled: self.is_enabled(ctx.save_state),
            selected: self.button_selected(ctx),
            display: self.button_display(free_selected),
        }
    }

    /// 是否可点（不碰 i18n，observer 里也能用）
    pub fn is_enabled(&self, save_state: &SaveState) -> bool {
        match self {
            Self::SelectPuzzle(storage) => save_state
                .top_level_worlds()
                .iter()
                .any(|entry| entry.slot.puzzle == *storage),
            Self::SelectSolution(storage) => save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.slot.solution.as_deref() == Some(storage.as_str())),
            Self::NewPuzzle | Self::NewFree | Self::Back => true,
            Self::NewSolution => {
                selected_top_level_kind(save_state) == Some(SaveKind::Puzzle)
            }
            Self::EditSelectedPuzzle => {
                selected_top_level_kind(save_state) == Some(SaveKind::Puzzle)
            }
            Self::RenameSelectedPuzzle | Self::DeleteSelectedPuzzle => {
                save_state.selected_puzzle.is_some()
                    && selected_top_level_kind(save_state)
                        .is_some_and(|kind| matches!(kind, SaveKind::Puzzle | SaveKind::Free))
            }
            Self::RenameSelectedSolution | Self::DeleteSelectedSolution => {
                !selected_is_free(save_state) && save_state.selected_solution.is_some()
            }
            Self::StartGame => match selected_top_level_kind(save_state) {
                Some(SaveKind::Free) => save_state.selected_puzzle.is_some(),
                Some(SaveKind::Puzzle) => save_state.selected_solution.is_some(),
                _ => false,
            },
        }
    }

    fn button_label(&self, ctx: &SaveListViewCtx<'_>) -> String {
        let save_state = ctx.save_state;
        match self {
            Self::SelectPuzzle(storage) => top_level_display_label(save_state, storage),
            Self::SelectSolution(storage) => solution_display_name(save_state, storage),
            Self::NewPuzzle => i18n.t("button.new_puzzle"),
            Self::NewFree => i18n.t("button.new_free"),
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

    fn button_display(&self, free_selected: bool) -> Option<Display> {
        match self {
            Self::EditSelectedPuzzle
            | Self::RenameSelectedSolution
            | Self::DeleteSelectedSolution
            | Self::NewSolution => free_selected.then_some(Display::None),
            _ => None,
        }
    }
}

/// 当前选中顶层世界的种类
pub fn selected_top_level_kind(save_state: &SaveState) -> Option<SaveKind> {
    let name = save_state.selected_puzzle.as_deref()?;
    save_state
        .top_level_worlds()
        .into_iter()
        .find(|entry| entry.slot.puzzle == name)
        .map(|entry| entry.kind)
}

fn selected_is_free(save_state: &SaveState) -> bool {
    selected_top_level_kind(save_state) == Some(SaveKind::Free)
}

fn top_level_display_label(save_state: &SaveState, storage: &str) -> String {
    let Some(entry) = save_state
        .top_level_worlds()
        .into_iter()
        .find(|entry| entry.slot.puzzle == storage)
    else {
        return storage.to_string();
    };
    let kind_key = match entry.kind {
        SaveKind::Puzzle => "save.kind.puzzle",
        SaveKind::Free => "save.kind.free",
        SaveKind::Solution => "save.kind.solution",
    };
    format!("{} · {}", entry.name, i18n.t(kind_key))
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
        .top_level_worlds()
        .into_iter()
        .map(|entry| entry.slot.puzzle.clone())
        .collect()
}

pub fn save_list_title() -> String {
    i18n.t("save.title.play_solution")
}
