use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, SimulationState, SolutionState, WorldEntryMode,
};
use crate::game::ui::{CarriedItem, InventoryItems};
use crate::game::world::grid::seed_demo_world;
use crate::shared::save::{load_world, save_solution_with_puzzle, save_world, SaveKind, SaveState};

use super::messages::{CreateNewPuzzle, CreateNewSolution, LoadWorld};
use super::world_access::PlayingWorldParams;
use super::world_ops::load_world_into_session;

pub fn handle_load_world(
    mut requests: MessageReader<LoadWorld>,
    mut world: PlayingWorldParams,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    for request in requests.read() {
        load_world_into_session(
            &request.name,
            request.entry,
            &mut world.world,
            &mut builder_mode,
            &mut inventory,
            &mut carried,
            &mut placement,
            &mut save_state,
            &mut solution_state,
            &mut simulation,
            &mut world.commands,
            &mut world.meshes,
            &world.block_entities,
            world.render_assets.as_deref(),
            &world.debug,
            &mut world.factory_structures,
            &mut world.movement_influence,
            &mut world.pusher_state,
            *mode.get(),
            &mut next_state,
        );
    }
}

pub fn handle_create_new_puzzle(
    mut requests: MessageReader<CreateNewPuzzle>,
    mut world: PlayingWorldParams,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    for request in requests.read() {
        world.world.clear();
        seed_demo_world(&mut world.world);
        *inventory = InventoryItems::for_mode(BuilderMode::Edit);
        if save_world(&world.world, &request.name, SaveKind::Puzzle, &inventory) {
            save_state.refresh();
            load_world_into_session(
                &request.name,
                WorldEntryMode::EditPuzzle,
                &mut world.world,
                &mut builder_mode,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut save_state,
                &mut solution_state,
                &mut simulation,
                &mut world.commands,
                &mut world.meshes,
                &world.block_entities,
                world.render_assets.as_deref(),
                &world.debug,
                &mut world.factory_structures,
                &mut world.movement_influence,
                &mut world.pusher_state,
                *mode.get(),
                &mut next_state,
            );
        }
    }
}

pub fn handle_create_new_solution(
    mut requests: MessageReader<CreateNewSolution>,
    mut world: PlayingWorldParams,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    for request in requests.read() {
        let Some(loaded) = load_world(&mut world.world, &request.puzzle) else {
            continue;
        };
        let puzzle_snapshot = loaded
            .puzzle_snapshot
            .unwrap_or_else(|| world.world.clone());
        *world.world = puzzle_snapshot.clone();
        *inventory = InventoryItems::for_mode(BuilderMode::Play);
        if save_solution_with_puzzle(
            &world.world,
            &request.name,
            &puzzle_snapshot,
            &inventory,
        ) {
            save_state.refresh();
            load_world_into_session(
                &request.name,
                WorldEntryMode::PlaySolution,
                &mut world.world,
                &mut builder_mode,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut save_state,
                &mut solution_state,
                &mut simulation,
                &mut world.commands,
                &mut world.meshes,
                &world.block_entities,
                world.render_assets.as_deref(),
                &world.debug,
                &mut world.factory_structures,
                &mut world.movement_influence,
                &mut world.pusher_state,
                *mode.get(),
                &mut next_state,
            );
        }
    }
}
