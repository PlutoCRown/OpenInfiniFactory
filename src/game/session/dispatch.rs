//! Imperative session API — upper layers call these without importing message types
//! or registering handlers in [`SessionPlugin`].

use bevy::prelude::*;

use crate::game::state::WorldEntryMode;
use crate::game::state::{SimulationState, SolutionState};
use crate::game::ui::InventoryItems;
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{SaveSlot, SaveState};

use super::messages::{
    CreateNewPuzzle, CreateNewSolution, ExitToMainMenu, LoadWorld, ResetSolution,
    SaveCurrentWorld, SaveCurrentWorldInvalidateSolutions, SaveWorldAsNewPuzzle, SwitchToEditMode,
};
use super::world_ops::{
    save_current_world as save_current_world_impl, save_current_world_invalidate_solutions,
    SaveCurrentWorldResult,
};

pub use super::world_ops::puzzle_save_needs_confirm;

pub fn save_current_world(commands: &mut Commands) {
    commands.queue(|world: &mut World| {
        save_current_world_in_world(world);
    });
}

#[allow(dead_code)]
pub fn save_world_as_new_puzzle(commands: &mut Commands, name: impl Into<String>) {
    let name = name.into();
    commands.queue(move |world: &mut World| {
        save_world_as_new_puzzle_in_world(world, name);
    });
}

pub fn exit_to_main_menu(commands: &mut Commands, save_first: bool) {
    commands.queue(move |world: &mut World| {
        exit_to_main_menu_in_world(world, save_first);
    });
}

#[allow(dead_code)]
pub fn reset_solution(commands: &mut Commands) {
    commands.queue(|world: &mut World| {
        reset_solution_in_world(world);
    });
}

#[allow(dead_code)]
pub fn switch_to_edit_mode(commands: &mut Commands, save_first: bool) {
    commands.queue(move |world: &mut World| {
        switch_to_edit_mode_in_world(world, save_first);
    });
}

pub fn load_world(commands: &mut Commands, slot: SaveSlot, entry: WorldEntryMode) {
    commands.queue(move |world: &mut World| {
        load_world_in_world(world, slot, entry);
    });
}

#[allow(dead_code)]
pub fn create_new_puzzle(commands: &mut Commands, name: impl Into<String>) {
    let name = name.into();
    commands.queue(move |world: &mut World| {
        create_new_puzzle_in_world(world, name);
    });
}

#[allow(dead_code)]
pub fn create_new_solution(
    commands: &mut Commands,
    name: impl Into<String>,
    puzzle: impl Into<String>,
) {
    let name = name.into();
    let puzzle = puzzle.into();
    commands.queue(move |world: &mut World| {
        create_new_solution_in_world(world, name, puzzle);
    });
}

pub fn save_current_world_in_world(world: &mut World) {
    world.write_message(SaveCurrentWorld);
}

pub fn save_world_as_new_puzzle_in_world(world: &mut World, name: String) {
    world.write_message(SaveWorldAsNewPuzzle { name });
}

pub fn exit_to_main_menu_in_world(world: &mut World, save_first: bool) {
    world.write_message(ExitToMainMenu { save_first });
}

pub fn reset_solution_in_world(world: &mut World) {
    world.write_message(ResetSolution);
}

pub fn switch_to_edit_mode_in_world(world: &mut World, save_first: bool) {
    world.write_message(SwitchToEditMode { save_first });
}

pub fn load_world_in_world(world: &mut World, slot: SaveSlot, entry: WorldEntryMode) {
    world.write_message(LoadWorld { slot, entry });
}

pub fn create_new_puzzle_in_world(world: &mut World, name: String) {
    world.write_message(CreateNewPuzzle { name });
}

pub fn create_new_solution_in_world(world: &mut World, name: String, puzzle: String) {
    world.write_message(CreateNewSolution { name, puzzle });
}

pub fn save_current_world_resources(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &SimulationState,
    player: Option<crate::shared::save::PlayerSave>,
) -> SaveCurrentWorldResult {
    save_current_world_impl(
        world,
        inventory,
        save_state,
        solution_state,
        simulation,
        player,
    )
}

pub fn save_current_world_invalidate_resources(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &SimulationState,
    player: Option<crate::shared::save::PlayerSave>,
) -> bool {
    save_current_world_invalidate_solutions(
        world,
        inventory,
        save_state,
        solution_state,
        simulation,
        player,
    )
}

pub fn save_current_world_invalidate_in_world(world: &mut World) {
    world.write_message(SaveCurrentWorldInvalidateSolutions);
}
