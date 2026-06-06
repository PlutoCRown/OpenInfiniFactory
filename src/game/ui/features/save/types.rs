use bevy::prelude::*;

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

#[derive(Component)]
pub struct SaveListTitleText;
