use bevy::prelude::*;

use crate::game::state::{
    GameMode, PlacementState, SimulationState, SolutionState, StartMenuScreen,
};
use crate::game::ui::InventoryItems;
use crate::shared::save::SaveState;

use super::messages::ExitToMainMenu;
use super::world_access::PlayingWorldParams;
use super::world_ops::{exit_to_main_menu, save_current_world};

pub fn handle_exit_to_main_menu(
    mut requests: MessageReader<ExitToMainMenu>,
    mut world: PlayingWorldParams,
    inventory: Res<InventoryItems>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
) {
    for request in requests.read() {
        if request.save_first {
            save_current_world(
                &world.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
                &simulation,
            );
        }
        exit_to_main_menu(
            &mut world.world,
            &mut placement,
            &mut save_state,
            &mut solution_state,
            &mut simulation,
            &mut world.commands,
            &mut world.meshes,
            world.render_assets.as_deref(),
            &world.block_entities,
            &world.debug,
            &mut world.structure_state,
            &mut world.movement_influence,
            &mut world.pusher_state,
            &mut next_state,
            &mut start_menu_screen,
            &mut world.block_index,
        );
    }
}
