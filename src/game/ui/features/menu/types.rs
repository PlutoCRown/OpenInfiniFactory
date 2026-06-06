use bevy::prelude::*;

use crate::game::state::SolutionState;
use crate::game::ui::core::action::UiActionLabel;
use crate::shared::i18n::I18n;
use crate::shared::save::{SaveKind, SaveState};

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum MenuAction {
    EditPuzzle,
    Play,
    Quit,
    Resume,
    ToggleBuilderMode,
    SaveWorld,
    SaveAsNewPuzzle,
    ResetSolution,
    OpenSettings,
    BackToMainMenu,
}

impl UiActionLabel for MenuAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::EditPuzzle => "button.edit_puzzle",
            Self::Play => "button.start_playing",
            Self::Quit => "button.quit_game",
            Self::Resume => "button.resume",
            Self::ToggleBuilderMode => "button.toggle_builder_mode",
            Self::SaveWorld => "button.save_world",
            Self::SaveAsNewPuzzle => "button.save_as_new_puzzle",
            Self::ResetSolution => "button.reset_solution",
            Self::OpenSettings => "button.settings",
            Self::BackToMainMenu => "button.back_to_main_menu",
        }
    }
}

impl MenuAction {
    pub fn label(self, save_state: &SaveState, i18n: &I18n) -> String {
        match self {
            Self::SaveWorld => match save_state.current_kind {
                Some(SaveKind::Solution) => i18n.text("button.save_solution"),
                _ => i18n.text("button.save_puzzle"),
            },
            _ => i18n.text(self.label_key()),
        }
    }

    pub fn pause_menu_visible(
        self,
        save_state: &SaveState,
        solution_state: &SolutionState,
    ) -> bool {
        use crate::game::state::WorldEntryMode;
        match self {
            Self::ToggleBuilderMode => solution_state.entry != WorldEntryMode::PlaySolution,
            Self::SaveAsNewPuzzle => save_state.current_kind == Some(SaveKind::Puzzle),
            Self::ResetSolution => save_state.current_kind == Some(SaveKind::Solution),
            _ => true,
        }
    }
}
