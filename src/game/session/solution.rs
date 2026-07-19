use bevy::prelude::*;

use crate::game::edit_history::EditHistory;
use crate::game::player::controller::{FlyCamera, capture_player_save};

use super::messages::{ResetSolution, SwitchToEditMode};
use super::world_access::{PlayingWorldParams, SessionStateParams};
use super::world_ops::{
    reset_current_solution, save_current_world, switch_to_edit_mode_and_rebuild,
};

pub fn handle_reset_solution(
    mut requests: MessageReader<ResetSolution>,
    mut playing: PlayingWorldParams,
    mut session: SessionStateParams,
    mut edit_history: ResMut<EditHistory>,
) {
    for _ in requests.read() {
        edit_history.clear();
        reset_current_solution(&mut playing, &mut session);
        session.playing_ui.paused = true;
    }
}

pub fn handle_switch_to_edit_mode(
    mut requests: MessageReader<SwitchToEditMode>,
    mut playing: PlayingWorldParams,
    mut session: SessionStateParams,
    player: Query<(&FlyCamera, &Transform)>,
    mut edit_history: ResMut<EditHistory>,
) {
    for request in requests.read() {
        if request.save_first {
            let player_save = player
                .single()
                .ok()
                .map(|(camera, transform)| capture_player_save(camera, transform));
            save_current_world(
                &playing.world,
                &session.inventory,
                &mut session.save_state,
                &mut session.solution_state,
                &session.simulation,
                player_save,
            );
        }
        edit_history.clear();
        switch_to_edit_mode_and_rebuild(&mut playing, &mut session);
    }
}
