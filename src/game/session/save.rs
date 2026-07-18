use bevy::prelude::*;

use crate::game::player::controller::{FlyCamera, capture_player_save};
use crate::game::state::{SimulationState, SolutionState};
use crate::game::ui::InventoryItems;
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{SaveKind, SaveState, save_puzzle_as};

use super::busy::SessionBusy;
#[cfg(not(target_arch = "wasm32"))]
use super::cover::begin_cover_capture;
use super::cover::should_capture_cover;
use super::messages::{
    SaveCurrentWorld, SaveCurrentWorldInvalidateSolutions, SaveWorldAsNewPuzzle,
};
use super::world_ops::{save_current_world, save_current_world_invalidate_solutions};

#[cfg(not(target_arch = "wasm32"))]
use crate::game::cameras::GameplayViewImage;

/// 排队中的显式保存（先亮「保存中」，下一帧再写盘）
#[derive(Resource, Default)]
pub struct PendingSave {
    pub hold_frames: u8,
    pub invalidate_solutions: bool,
    pub active: bool,
}

pub fn handle_save_current_world(
    mut requests: MessageReader<SaveCurrentWorld>,
    mut busy: ResMut<SessionBusy>,
    mut pending: ResMut<PendingSave>,
) {
    for _ in requests.read() {
        *busy = SessionBusy::Saving;
        pending.active = true;
        pending.invalidate_solutions = false;
        pending.hold_frames = 1;
    }
}

pub fn handle_save_current_world_invalidate_solutions(
    mut requests: MessageReader<SaveCurrentWorldInvalidateSolutions>,
    mut busy: ResMut<SessionBusy>,
    mut pending: ResMut<PendingSave>,
) {
    for _ in requests.read() {
        *busy = SessionBusy::Saving;
        pending.active = true;
        pending.invalidate_solutions = true;
        pending.hold_frames = 1;
    }
}

/// 真正执行排队中的保存，并在成功后更新封面
pub fn process_pending_save(
    mut pending: ResMut<PendingSave>,
    mut busy: ResMut<SessionBusy>,
    world: Res<WorldBlocks>,
    inventory: Res<InventoryItems>,
    player: Query<(&FlyCamera, &Transform)>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    simulation: Res<SimulationState>,
    mut commands: Commands,
    #[cfg(not(target_arch = "wasm32"))] view_image: Option<Res<GameplayViewImage>>,
) {
    if !pending.active {
        return;
    }
    if pending.hold_frames > 0 {
        pending.hold_frames -= 1;
        return;
    }
    pending.active = false;
    let invalidate = pending.invalidate_solutions;
    let player_save = player
        .single()
        .ok()
        .map(|(camera, transform)| capture_player_save(camera, transform));
    let saved = if invalidate {
        save_current_world_invalidate_solutions(
            &world,
            &inventory,
            &mut save_state,
            &mut solution_state,
            &simulation,
            player_save,
        )
    } else {
        matches!(
            save_current_world(
                &world,
                &inventory,
                &mut save_state,
                &mut solution_state,
                &simulation,
                player_save,
            ),
            crate::game::session::SaveCurrentWorldResult::Saved
        )
    };
    if saved && should_capture_cover(save_state.current.as_ref()) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(view_image) = view_image.as_ref() {
            begin_cover_capture(
                &mut commands,
                save_state.current.as_ref().unwrap(),
                view_image,
            );
        }
    }
    *busy = SessionBusy::None;
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
        if let Some(slot) = save_puzzle_as(snapshot, &request.name, &inventory, player_save) {
            save_state.current = Some(slot);
            save_state.current_kind = Some(SaveKind::Puzzle);
            solution_state.dirty = false;
            solution_state.puzzle_id = None;
            solution_state.puzzle_snapshot = None;
            save_state.refresh();
        }
    }
}
