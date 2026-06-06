use bevy::prelude::*;

use crate::game::ui::core::action::UiActionLabel;

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
