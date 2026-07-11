use bevy::prelude::*;

use crate::game::player::controller::{capture_player_save, FlyCamera};
use crate::game::state::{
    GameMode, PlacementState, SimulationState, SolutionState, StartMenuScreen,
};
use crate::game::ui::InventoryItems;
use crate::shared::save::SaveState;

use super::cover::{
    begin_cover_capture, should_capture_cover, CoverScreenshotComplete, PendingMainMenuExit,
};
use super::messages::ExitToMainMenu;
use super::world_access::PlayingWorldParams;
use super::world_ops::{exit_to_main_menu, save_current_world};

#[cfg(not(target_arch = "wasm32"))]
use crate::game::cameras::GameplayViewImage;

pub fn handle_exit_to_main_menu(
    mut requests: MessageReader<ExitToMainMenu>,
    mut world: PlayingWorldParams,
    inventory: Res<InventoryItems>,
    player: Query<(&FlyCamera, &Transform)>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut pending_exit: ResMut<PendingMainMenuExit>,
    #[cfg(not(target_arch = "wasm32"))] view_image: Option<Res<GameplayViewImage>>,
) {
    for request in requests.read() {
        let player_save = player
            .single()
            .ok()
            .map(|(camera, transform)| capture_player_save(camera, transform));
        if request.save_first {
            save_current_world(
                &world.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
                &simulation,
                player_save.clone(),
            );
        }
        if should_capture_cover(save_state.current.as_deref()) {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(view_image) = view_image.as_ref() {
                begin_cover_capture(
                    &mut world.commands,
                    save_state.current.as_ref().unwrap(),
                    view_image,
                );
                pending_exit.0 = true;
                continue;
            }
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

#[cfg(not(target_arch = "wasm32"))]
pub fn finish_pending_main_menu_exit(
    mut complete: MessageReader<CoverScreenshotComplete>,
    mut world: PlayingWorldParams,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut pending_exit: ResMut<PendingMainMenuExit>,
) {
    if !pending_exit.0 {
        return;
    }
    for _ in complete.read() {
        pending_exit.0 = false;
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
