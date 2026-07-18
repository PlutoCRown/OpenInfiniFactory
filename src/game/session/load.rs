use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};

use crate::game::edit_history::EditHistory;
use crate::game::state::{
    BuilderMode, GameMode, PendingPlayerSpawn, PlacementState, SimulationState, SolutionState,
    WorldEntryMode,
};
use crate::game::ui::{CarriedItem, InventoryItems};
use crate::shared::save::{
    LoadedSave, SaveSlot, SaveState, create_puzzle_from_default_template, decode_save_slot,
    load_world, save_solution_as,
};

use super::busy::SessionBusy;
use super::messages::{CreateNewPuzzle, CreateNewSolution, LoadWorld};
use super::world_access::PlayingWorldParams;
use super::world_ops::load_world_into_session;

/// 后台解码中的读档请求
#[derive(Resource, Default)]
pub struct PendingWorldLoad {
    task: Option<Task<Option<(SaveSlot, WorldEntryMode, LoadedSave)>>>,
    /// `None`：尚未出结果；`Some(None)`：解码失败；`Some(Some(..))`：成功
    ready: Option<Option<(SaveSlot, WorldEntryMode, LoadedSave)>>,
    /// 至少留一帧给「加载中」绘制
    hold_frames: u8,
}

pub fn handle_load_world(
    mut requests: MessageReader<LoadWorld>,
    mut pending: ResMut<PendingWorldLoad>,
    mut edit_history: ResMut<EditHistory>,
    mut busy: ResMut<SessionBusy>,
) {
    for request in requests.read() {
        edit_history.clear();
        *busy = SessionBusy::Loading;
        pending.hold_frames = 1;
        pending.ready = None;
        let slot = request.slot.clone();
        let entry = request.entry;
        pending.task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { decode_save_slot(&slot).map(|loaded| (slot, entry, loaded)) }),
        );
    }
}

pub fn poll_pending_world_load(
    mut pending: ResMut<PendingWorldLoad>,
    mut world: PlayingWorldParams,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut pending_player: ResMut<PendingPlayerSpawn>,
    mut busy: ResMut<SessionBusy>,
    mut scene_registry: ResMut<crate::game::scene_blocks::SceneBlockRegistry>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    if let Some(mut task) = pending.task.take() {
        match block_on(future::poll_once(&mut task)) {
            Some(result) => {
                pending.ready = Some(result);
            }
            None => {
                pending.task = Some(task);
            }
        }
    }

    if pending.hold_frames > 0 {
        pending.hold_frames -= 1;
        return;
    }

    if pending.task.is_some() {
        return;
    }

    let Some(outcome) = pending.ready.take() else {
        return;
    };
    let Some((slot, entry, loaded)) = outcome else {
        bevy::log::warn!("Failed to decode save");
        *busy = SessionBusy::None;
        return;
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::shared::platform::SAVE_DIR;
        let puzzle_dir = std::path::PathBuf::from(SAVE_DIR).join(&slot.puzzle);
        if let Err(err) =
            crate::game::scene_blocks::merge_puzzle_scene_blocks(&mut scene_registry, &puzzle_dir)
        {
            bevy::log::error!("puzzle scene blocks: {err}");
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = &mut scene_registry;
    }

    load_world_into_session(
        &slot,
        entry,
        loaded,
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
        &mut world.structure_state,
        &mut world.movement_influence,
        &mut world.pusher_state,
        &mut pending_player,
        *mode.get(),
        &mut next_state,
        &mut world.block_index,
    );
    *busy = SessionBusy::None;
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
    mut pending_player: ResMut<PendingPlayerSpawn>,
    mut edit_history: ResMut<EditHistory>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    for request in requests.read() {
        let Some(slot) = create_puzzle_from_default_template(&request.name) else {
            bevy::log::warn!("failed to create puzzle `{}` from template", request.name);
            continue;
        };
        save_state.refresh();
        edit_history.clear();
        let Some(loaded) = decode_save_slot(&slot) else {
            continue;
        };
        load_world_into_session(
            &slot,
            WorldEntryMode::EditPuzzle,
            loaded,
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
            &mut world.structure_state,
            &mut world.movement_influence,
            &mut world.pusher_state,
            &mut pending_player,
            *mode.get(),
            &mut next_state,
            &mut world.block_index,
        );
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
    mut pending_player: ResMut<PendingPlayerSpawn>,
    mut edit_history: ResMut<EditHistory>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    for request in requests.read() {
        let puzzle_slot = SaveSlot::puzzle(&request.puzzle);
        let Some(loaded) = load_world(&mut world.world, &puzzle_slot) else {
            continue;
        };
        *world.world = loaded.world;
        *inventory = InventoryItems::for_mode(BuilderMode::Play);
        let Some(solution_slot) = save_solution_as(
            &world.world,
            &request.puzzle,
            &request.name,
            &inventory,
            None,
        ) else {
            continue;
        };
        save_state.refresh();
        edit_history.clear();
        let Some(loaded) = decode_save_slot(&solution_slot) else {
            continue;
        };
        load_world_into_session(
            &solution_slot,
            WorldEntryMode::PlaySolution,
            loaded,
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
            &mut world.structure_state,
            &mut world.movement_influence,
            &mut world.pusher_state,
            &mut pending_player,
            *mode.get(),
            &mut next_state,
            &mut world.block_index,
        );
    }
}
