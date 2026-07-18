use bevy::prelude::*;

#[derive(Component, Clone, Debug, Eq, PartialEq)]
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

/// 谜题列里动态行的容器（新建按钮不在这里，避免重建时被清掉）
#[derive(Component, Clone, Copy)]
pub struct SaveListPuzzleRows;

/// 方案列里动态行的容器
#[derive(Component, Clone, Copy)]
pub struct SaveListSolutionRows;

/// 列顶部的「新建」按钮（稳定实体）
#[derive(Component, Clone, Copy)]
pub struct SaveListCreateButton;

#[derive(Component)]
pub struct SaveListPrompt;

#[derive(Resource, Default)]
pub struct SaveListRenderState {
    pub entry: Option<crate::game::state::WorldEntryMode>,
    pub puzzle_keys: Vec<String>,
    pub solution_keys: Vec<String>,
    /// 行重建后下一帧再刷按钮样式/标签（Commands 延迟生效）
    pub paint_buttons: bool,
}

#[derive(Component)]
pub struct SaveListTitleText;
