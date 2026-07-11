use bevy::prelude::*;

use crate::game::player::controller::{capture_player_save, FlyCamera};
use crate::game::state::{SimulationState, SolutionState};
use crate::game::ui::InventoryItems;
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{save_puzzle, SaveKind, SaveState};

use super::messages::{
    SaveCurrentWorld, SaveCurrentWorldInvalidateSolutions, SaveWorldAsNewPuzzle,
};
use super::world_ops::{save_current_world, save_current_world_invalidate_solutions};

pub fn handle_save_current_world(
    mut requests: MessageReader<SaveCurrentWorld>,
    world: Res<WorldBlocks>,
    inventory: Res<InventoryItems>,
    player: Query<(&FlyCamera, &Transform)>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    simulation: Res<SimulationState>,
) {
    for _ in requests.read() {
        let player_save = player
            .single()
            .ok()
            .map(|(camera, transform)| capture_player_save(camera, transform));
        let _ = save_current_world(
            &world,
            &inventory,
            &mut save_state,
            &mut solution_state,
            &simulation,
            player_save,
        );
    }
}

pub fn handle_save_current_world_invalidate_solutions(
    mut requests: MessageReader<SaveCurrentWorldInvalidateSolutions>,
    world: Res<WorldBlocks>,
    inventory: Res<InventoryItems>,
    player: Query<(&FlyCamera, &Transform)>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    simulation: Res<SimulationState>,
) {
    for _ in requests.read() {
        let player_save = player
            .single()
            .ok()
            .map(|(camera, transform)| capture_player_save(camera, transform));
        save_current_world_invalidate_solutions(
            &world,
            &inventory,
            &mut save_state,
            &mut solution_state,
            &simulation,
            player_save,
        );
    }
}

pub fn handle_save_world_as_new_puzzle(
    mut requests: MessageReader<SaveWorldAsNewPuzzle>,
    world: Res<WorldBlocks>,
    inventory: Res<InventoryItems>,
    player: Query<(&FlyCamera, &Transform)>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    simulation: Res<SimulationState>,
) {
    for request in requests.read() {
        let player_save = player
            .single()
            .ok()
            .map(|(camera, transform)| capture_player_save(camera, transform));
        let snapshot = simulation.authoring_world(&world);
        if save_puzzle(snapshot, &request.name, &inventory, player_save) {
            save_state.current = Some(request.name.clone());
            save_state.current_kind = Some(SaveKind::Puzzle);
            solution_state.dirty = false;
            solution_state.puzzle_id = None;
            solution_state.puzzle_snapshot = None;
            save_state.refresh();
        }
    }
}
