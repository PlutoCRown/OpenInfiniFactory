use bevy::prelude::*;

use crate::game::ui::core::action::UiActionLabel;

#[derive(Component, Clone)]
pub enum SaveListAction {
    NewPuzzle,
    NewSolution,
    LoadPuzzle(String),
    LoadSolution(String),
    RenamePuzzle(String),
    RenameSolution(String),
    DeletePuzzle(String),
    DeleteSolution(String),
    Back,
}

#[derive(Component)]
pub struct SaveListCloseButton;

#[derive(Component)]
pub struct SaveListPanel;

#[derive(Component, Clone, Copy)]
pub struct SaveListPuzzleColumn;

#[derive(Component, Clone, Copy)]
pub struct SaveListSolutionColumn;

#[derive(Component)]
pub struct SaveListPrompt;

#[derive(Resource, Default)]
pub struct SaveListRenderState {
    pub entry: Option<crate::game::state::WorldEntryMode>,
    pub puzzle_keys: Vec<String>,
    pub solution_keys: Vec<String>,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum TextPromptAction {
    Confirm,
    Cancel,
}

#[derive(Component)]
pub struct TextPromptRoot;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum TextPromptText {
    Title,
    Value,
}

#[derive(Clone, Eq, PartialEq)]
pub enum TextPromptKind {
    NewPuzzle,
    NewSolution { puzzle: String },
    RenamePuzzle { name: String },
    RenameSolution { name: String },
    SaveAsNewPuzzle,
}

#[derive(Resource, Default)]
pub struct TextPromptState {
    pub kind: Option<TextPromptKind>,
    pub value: String,
}

#[derive(Component, Clone, Copy)]
pub enum ConfirmDialogAction {
    Primary,
    Secondary,
    Cancel,
}

impl UiActionLabel for ConfirmDialogAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::Primary => "button.confirm",
            Self::Secondary => "button.confirm",
            Self::Cancel => "button.cancel",
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum ConfirmDialogKind {
    DeleteSave { name: String },
    ResetSolution,
    ReturnToMain,
    SaveSolutionBeforeEdit,
}

#[derive(Resource, Default)]
pub struct ConfirmDialogState {
    pub kind: Option<ConfirmDialogKind>,
}
