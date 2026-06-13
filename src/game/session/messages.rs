use bevy::prelude::*;

use crate::game::state::WorldEntryMode;

#[derive(Clone, Message)]
pub struct SaveCurrentWorld;

#[derive(Clone, Message)]
pub struct SaveCurrentWorldInvalidateSolutions;

#[derive(Clone, Message)]
pub struct SaveWorldAsNewPuzzle {
    pub name: String,
}

#[derive(Clone, Message)]
pub struct ExitToMainMenu {
    pub save_first: bool,
}

#[derive(Clone, Message)]
pub struct ResetSolution;

#[derive(Clone, Message)]
pub struct SwitchToEditMode {
    pub save_first: bool,
}

#[derive(Clone, Message)]
pub struct LoadWorld {
    pub name: String,
    pub entry: WorldEntryMode,
}

#[derive(Clone, Message)]
pub struct CreateNewPuzzle {
    pub name: String,
}

#[derive(Clone, Message)]
pub struct CreateNewSolution {
    pub name: String,
    pub puzzle: String,
}
