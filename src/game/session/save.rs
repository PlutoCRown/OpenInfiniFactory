use bevy::prelude::*;

use crate::game::state::{SimulationState, SolutionState};
use crate::game::ui::InventoryItems;
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{save_world, SaveKind, SaveState};

use super::messages::{SaveCurrentWorld, SaveWorldAsNewPuzzle};
use super::world_ops::save_current_world;

pub fn handle_save_current_world(
    mut requests: MessageReader<SaveCurrentWorld>,
    world: Res<WorldBlocks>,
    inventory: Res<InventoryItems>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    simulation: Res<SimulationState>,
) {
    for _ in requests.read() {
        save_current_world(
            &world,
            &inventory,
            &mut save_state,
            &mut solution_state,
            &simulation,
        );
    }
}

pub fn handle_save_world_as_new_puzzle(
    mut requests: MessageReader<SaveWorldAsNewPuzzle>,
    world: Res<WorldBlocks>,
    inventory: Res<InventoryItems>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    simulation: Res<SimulationState>,
) {
    for request in requests.read() {
        let snapshot = simulation.authoring_world(&world);
        if save_world(snapshot, &request.name, SaveKind::Puzzle, &inventory) {
            save_state.current = Some(request.name.clone());
            save_state.current_kind = Some(SaveKind::Puzzle);
            solution_state.dirty = false;
            save_state.refresh();
        }
    }
}
