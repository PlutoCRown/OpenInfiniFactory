use bevy::prelude::*;

use crate::game::player::controller::{capture_player_save, FlyCamera};
use crate::game::state::{
    BuilderMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
};
use crate::game::ui::{CarriedItem, InventoryItems};
use crate::shared::save::SaveState;

use super::messages::{ResetSolution, SwitchToEditMode};
use super::world_access::PlayingWorldParams;
use super::world_ops::{
    reset_current_solution, save_current_world, switch_to_edit_mode_and_rebuild,
};

pub fn handle_reset_solution(
    mut requests: MessageReader<ResetSolution>,
    mut world: PlayingWorldParams,
    mut simulation: ResMut<SimulationState>,
    solution_state: Res<SolutionState>,
    mut playing_ui: ResMut<PlayingUiState>,
) {
    for _ in requests.read() {
        reset_current_solution(
            &mut world.world,
            &mut simulation,
            &mut world.commands,
            &mut world.meshes,
            &world.block_entities,
            world.render_assets.as_deref(),
            &world.debug,
            &mut world.structure_state,
            &mut world.movement_influence,
            &mut world.pusher_state,
            &solution_state,
            &mut world.block_index,
        );
        playing_ui.paused = true;
    }
}

pub fn handle_switch_to_edit_mode(
    mut requests: MessageReader<SwitchToEditMode>,
    mut world: PlayingWorldParams,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut playing_ui: ResMut<PlayingUiState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    player: Query<(&FlyCamera, &Transform)>,
    simulation: Res<SimulationState>,
) {
    for request in requests.read() {
        if request.save_first {
            let player_save = player
                .single()
                .ok()
                .map(|(camera, transform)| capture_player_save(camera, transform));
            save_current_world(
                &world.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
                &simulation,
                player_save,
            );
        }
        switch_to_edit_mode_and_rebuild(
            &mut world.world,
            &mut builder_mode,
            &mut inventory,
            &mut carried,
            &mut placement,
            &mut playing_ui,
            &mut save_state,
            &mut solution_state,
            &mut world.commands,
            &mut world.meshes,
            &world.block_entities,
            world.render_assets.as_deref(),
            &world.debug,
            &mut world.structure_state,
            &mut world.movement_influence,
            &mut world.pusher_state,
            &mut world.block_index,
        );
    }
}
