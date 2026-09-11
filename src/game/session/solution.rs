use bevy::prelude::*;

use crate::game::edit_history::EditHistory;
use crate::game::player::controller::{FlyCamera, capture_player_save};

use super::messages::{BeginSolutionPlay, ResetSolution, SwitchToEditMode};
use super::world_access::{PlayingWorldParams, SessionStateParams};
use super::world_ops::{
    reset_current_solution, save_current_world, switch_to_edit_mode_and_rebuild,
};
use crate::game::state::{BuilderMode, WorldEntryMode};
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{SaveKind, SaveSlot, next_named_save, solution_names_for_puzzle};

/// 将编辑中的谜题投影为新的方案游玩会话。
pub fn handle_begin_solution_play(
    mut requests: MessageReader<BeginSolutionPlay>,
    world: Res<WorldBlocks>,
    mut session: SessionStateParams,
) {
    for _ in requests.read() {
        if session.solution_state.entry != WorldEntryMode::EditPuzzle
            || *session.builder_mode != BuilderMode::Edit
        {
            continue;
        }
        session.simulation.reset();
        session.solution_state.puzzle_snapshot = Some(world.clone());
        session.solution_state.puzzle_id = session
            .save_state
            .current
            .as_ref()
            .map(|slot| slot.puzzle.clone());
        let puzzle_id = session
            .save_state
            .current
            .as_ref()
            .map(|slot| slot.puzzle.clone())
            .unwrap_or_else(|| "solution".to_string());
        let solution_name = next_named_save(
            &solution_names_for_puzzle(&session.save_state.entries, &puzzle_id),
            "solution",
        );
        session.save_state.current = Some(SaveSlot::solution(&puzzle_id, &solution_name));
        session.save_state.current_kind = Some(SaveKind::Solution);
        let filter = session.solution_state.factory_block_filter.clone();
        session.inventory.begin_play_from_edit(filter.as_ref());
        session.pending_player.0 = session.solution_state.solution_spawn.clone();
        *session.builder_mode = BuilderMode::Play;
        session.carried.clear();
        session.placement.selected = 0;
        session.ui_navigation.close_pause();
    }
}

pub fn handle_reset_solution(
    mut requests: MessageReader<ResetSolution>,
    mut playing: PlayingWorldParams,
    mut session: SessionStateParams,
    mut edit_history: ResMut<EditHistory>,
) {
    for _ in requests.read() {
        edit_history.clear();
        reset_current_solution(&mut playing, &mut session);
        session.ui_navigation.open_pause();
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
