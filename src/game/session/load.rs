use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};

use crate::game::cameras::{GameplayCamera, PlayingUiCamera, UiCamera};
use crate::game::edit_history::EditHistory;
use crate::game::state::{BuilderMode, GameMode, WorldEntryMode};
use crate::game::ui::InventoryItems;
use crate::game::world::rendering::SceneFrameReady;
use crate::shared::save::{
    LoadedSave, SaveSlot, create_free_from_default_template, create_puzzle_from_default_template,
    decode_save_slot, save_solution_as,
};
use crate::sim_bridge::{SimulationPresentationState, reset_simulation_presentation};

use super::busy::{SessionBusy, SessionBusyCover};
use super::messages::{CreateNewFree, CreateNewPuzzle, CreateNewSolution, LoadWorld};
use super::world_access::{PlayingWorldParams, SessionStateParams};
use super::world_ops::load_world_into_session;

/// 后台解码中的读档请求
#[derive(Resource, Default)]
pub struct PendingWorldLoad {
    task: Option<Task<Option<(SaveSlot, WorldEntryMode, LoadedSave)>>>,
    /// `None`：尚未出结果；`Some(None)`：解码失败；`Some(Some(..))`：成功
    ready: Option<Option<(SaveSlot, WorldEntryMode, LoadedSave)>>,
    /// 进 Playing 的 OnEnter 重建完成后再清 SessionBusy
    release_busy_after_playing: bool,
    /// 超时仅用于报告渲染失败，不作为正常放行条件。
    render_started: f64,
    /// 先等忙碌遮罩真正绘制一次，再在主线程应用解码后的世界。
    cover_frame: Option<SceneFrameReady>,
}

pub fn handle_load_world(
    mut requests: MessageReader<LoadWorld>,
    mut pending: ResMut<PendingWorldLoad>,
    mut edit_history: ResMut<EditHistory>,
    mut busy: ResMut<SessionBusy>,
    time: Res<Time<Real>>,
    cameras: Query<(Entity, &Camera), Or<(With<UiCamera>, With<PlayingUiCamera>)>>,
    mut commands: Commands,
) {
    for request in requests.read() {
        edit_history.clear();
        *busy = SessionBusy::Loading;
        pending.ready = None;
        pending.release_busy_after_playing = false;
        pending.render_started = time.elapsed_secs_f64();
        pending.cover_frame = None;
        for (entity, camera) in &cameras {
            if camera.is_active {
                let frame = SceneFrameReady::default();
                commands.entity(entity).insert(frame.clone());
                pending.cover_frame = Some(frame);
                break;
            }
        }
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
    mut playing: PlayingWorldParams,
    mut session: SessionStateParams,
    mut busy: ResMut<SessionBusy>,
    mut cover: ResMut<SessionBusyCover>,
    mut scene_registry: ResMut<crate::game::scene_blocks::SceneBlockRegistry>,
    mode: Res<State<GameMode>>,
    time: Res<Time<Real>>,
    cameras: Query<Entity, With<GameplayCamera>>,
    ui_cameras: Query<Entity, Or<(With<UiCamera>, With<PlayingUiCamera>)>>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut presentation: ResMut<SimulationPresentationState>,
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

    if pending.task.is_some() {
        return;
    }

    if pending
        .cover_frame
        .as_ref()
        .is_some_and(|frame| !frame.is_ready() && !frame.is_failed())
        && time.elapsed_secs_f64() - pending.render_started <= 30.0
    {
        return;
    }

    let Some(outcome) = pending.ready.take() else {
        return;
    };
    if pending
        .cover_frame
        .as_ref()
        .is_some_and(|frame| !frame.is_ready())
    {
        bevy::log::warn!("loading cover failed to render; continuing with decoded world");
    }
    pending.cover_frame = None;
    for camera in &ui_cameras {
        playing.commands.entity(camera).remove::<SceneFrameReady>();
    }
    let Some((slot, entry, loaded)) = outcome else {
        bevy::log::warn!("Failed to decode save");
        *busy = SessionBusy::None;
        cover.clear();
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
        &mut playing,
        &mut session,
        &slot,
        entry,
        loaded,
        *mode.get(),
        &mut next_state,
    );
    reset_simulation_presentation(&mut presentation);
    // 游玩中换档也使用独立回执，旧世界的 GPU 回调不能放行新世界。
    for camera in &cameras {
        playing
            .commands
            .entity(camera)
            .insert(SceneFrameReady::default());
    }
    pending.release_busy_after_playing = true;
    pending.render_started = time.elapsed_secs_f64();
}

/// 世界重建且 GPU 首帧完成后解除遮罩，渲染失败时记录原因。
pub fn release_session_busy_after_playing(
    mut pending: ResMut<PendingWorldLoad>,
    mut busy: ResMut<SessionBusy>,
    mut cover: ResMut<SessionBusyCover>,
    mode: Res<State<GameMode>>,
    frames: Query<&SceneFrameReady, With<GameplayCamera>>,
    time: Res<Time<Real>>,
) {
    if !pending.release_busy_after_playing || *mode.get() != GameMode::Playing {
        return;
    }
    let timed_out = time.elapsed_secs_f64() - pending.render_started > 30.0;
    let ready = frames.single().is_ok_and(|frame| frame.is_ready());
    let failed = frames.single().is_ok_and(|frame| frame.is_failed());
    if !ready && !failed && !timed_out {
        return;
    }
    if failed || timed_out {
        bevy::log::error!("scene first frame failed (GPU failure: {failed}, timeout: {timed_out})");
    }
    pending.release_busy_after_playing = false;
    if *busy == SessionBusy::Loading {
        *busy = SessionBusy::None;
    }
    cover.clear();
}

pub fn handle_create_new_puzzle(
    mut requests: MessageReader<CreateNewPuzzle>,
    mut playing: PlayingWorldParams,
    mut session: SessionStateParams,
    mut edit_history: ResMut<EditHistory>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut presentation: ResMut<SimulationPresentationState>,
) {
    for request in requests.read() {
        let Some(slot) = create_puzzle_from_default_template(&request.name) else {
            bevy::log::warn!("failed to create puzzle `{}` from template", request.name);
            continue;
        };
        // 马上进 Playing，不必 refresh 列表（会触发存档页无意义重建）
        edit_history.clear();
        let Some(loaded) = decode_save_slot(&slot) else {
            continue;
        };
        load_world_into_session(
            &mut playing,
            &mut session,
            &slot,
            WorldEntryMode::EditPuzzle,
            loaded,
            *mode.get(),
            &mut next_state,
        );
        reset_simulation_presentation(&mut presentation);
    }
}

pub fn handle_create_new_free(
    mut requests: MessageReader<CreateNewFree>,
    mut playing: PlayingWorldParams,
    mut session: SessionStateParams,
    mut edit_history: ResMut<EditHistory>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut presentation: ResMut<SimulationPresentationState>,
) {
    for request in requests.read() {
        let Some(slot) = create_free_from_default_template(&request.name) else {
            bevy::log::warn!("failed to create free `{}` from template", request.name);
            continue;
        };
        edit_history.clear();
        let Some(loaded) = decode_save_slot(&slot) else {
            continue;
        };
        load_world_into_session(
            &mut playing,
            &mut session,
            &slot,
            WorldEntryMode::Free,
            loaded,
            *mode.get(),
            &mut next_state,
        );
        reset_simulation_presentation(&mut presentation);
    }
}

pub fn handle_create_new_solution(
    mut requests: MessageReader<CreateNewSolution>,
    mut playing: PlayingWorldParams,
    mut session: SessionStateParams,
    mut edit_history: ResMut<EditHistory>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut presentation: ResMut<SimulationPresentationState>,
) {
    for request in requests.read() {
        let puzzle_slot = SaveSlot::puzzle(&request.puzzle);
        let Some(loaded) = decode_save_slot(&puzzle_slot) else {
            continue;
        };
        let inventory = InventoryItems::for_entry_with_filter(
            WorldEntryMode::PlaySolution,
            BuilderMode::Play,
            loaded.factory_block_filter.as_ref(),
        );
        let Some(solution_slot) = save_solution_as(
            &loaded.world,
            &request.puzzle,
            &request.name,
            &inventory.to_saved_hotbar(),
            None,
        ) else {
            continue;
        };
        // 马上进 Playing，不必 refresh 列表（会触发存档页无意义重建）
        edit_history.clear();
        let Some(loaded) = decode_save_slot(&solution_slot) else {
            continue;
        };
        load_world_into_session(
            &mut playing,
            &mut session,
            &solution_slot,
            WorldEntryMode::PlaySolution,
            loaded,
            *mode.get(),
            &mut next_state,
        );
        reset_simulation_presentation(&mut presentation);
    }
}
