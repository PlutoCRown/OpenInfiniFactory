use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::sync::{mpsc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::thread::{self, JoinHandle};

use crate::debug_http::protocol::{
    help_json, json_error, json_ok, DebugHttpCommand, DebugHttpRequest,
};
use crate::debug_http::snapshot::{
    block_json, cursor_target_json, perf_stats_json, pos_json, simulation_status_json,
};
use crate::debug_http::world_ops::{block_kinds_json, parse_block_kind, parse_facing, place_block};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::edit_history::EditHistory;
use crate::game::player::controller::FlyCamera;
use crate::game::session::PlayingWorldParams;
use crate::game::simulation::runtime::{
    PendingGeneratedMaterials, SignalNetworkCache, SimulationPresentationState, SimulationStepStats,
};
use crate::game::state::{BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState};
use crate::game::systems::perf::PerfStats;
use crate::game::systems::simulation_controls::{
    request_continuous_run, request_one_turn, start_simulation_if_needed,
};
use crate::game::ui::UiRuntime;
use crate::game::world::rendering::BlockEntity;
use crate::shared::launch::{LaunchOptions, DEFAULT_DEBUG_HTTP_PORT};
use crate::sim_core::{SimSnapshot, SimulationDebugLog, SimulationWorker, TurnCache};

#[derive(Resource)]
pub struct DebugHttpBridge {
    receiver: Mutex<mpsc::Receiver<DebugHttpRequest>>,
    #[cfg(not(target_arch = "wasm32"))]
    _thread: JoinHandle<()>,
    pub port: u16,
}

#[derive(Resource, Default)]
pub struct PendingDebugHttpStart(pub bool);

/// 供 HTTP /perf 读取的帧统计（合并参数，避免 poll 系统超限）
#[derive(SystemParam)]
pub struct DebugHttpPerfSnapshot<'w, 's> {
    perf: Res<'w, PerfStats>,
    diagnostics: Res<'w, DiagnosticsStore>,
    sim_stats: Res<'w, SimulationStepStats>,
    player: Query<'w, 's, &'static Transform, With<FlyCamera>>,
    block_entities: Query<'w, 's, Entity, With<BlockEntity>>,
}

impl<'w, 's> DebugHttpPerfSnapshot<'w, 's> {
    fn capture(
        &self,
        builder_mode: BuilderMode,
        simulation: &SimulationState,
        block_count: usize,
    ) -> serde_json::Value {
        let fps = self
            .diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
            .unwrap_or(0.0);
        let player_pos = self
            .player
            .single()
            .ok()
            .map(|transform| transform.translation);
        perf_stats_json(
            fps,
            &self.perf,
            &self.sim_stats,
            builder_mode,
            simulation,
            block_count,
            self.block_entities.iter().len(),
            player_pos,
        )
    }
}

pub struct DebugToolsPlugin;

impl Plugin for DebugToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationDebugLog>()
            .init_resource::<PendingDebugHttpStart>();

        #[cfg(not(target_arch = "wasm32"))]
        {
            if app
                .world()
                .get_resource::<LaunchOptions>()
                .is_some_and(LaunchOptions::debug_http_enabled)
            {
                app.add_systems(Startup, start_debug_http_server);
            }
            app.add_systems(Update, process_pending_debug_http_start);
            app.add_systems(
                Update,
                poll_debug_http
                    .before(crate::game::systems::simulation_controls::simulation_controls),
            );
        }

        app.add_systems(Update, sync_sim_debug_log);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_sim_debug_log(
    debug: Res<crate::game::systems::debug::DebugState>,
    http: Option<Res<DebugHttpBridge>>,
    mut sim_log: ResMut<SimulationDebugLog>,
) {
    sim_log.set_enabled(debug.enabled || http.is_some());
}

#[cfg(target_arch = "wasm32")]
fn sync_sim_debug_log(
    debug: Res<crate::game::systems::debug::DebugState>,
    mut sim_log: ResMut<SimulationDebugLog>,
) {
    sim_log.set_enabled(debug.enabled);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_debug_http_server(
    launch: Res<LaunchOptions>,
    mut commands: Commands,
    mut sim_log: ResMut<SimulationDebugLog>,
    bridge: Option<Res<DebugHttpBridge>>,
) {
    if bridge.is_some() {
        return;
    }
    let Some(port) = launch.debug_http_port else {
        return;
    };
    try_start_debug_http_server(&mut commands, &mut sim_log, port);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn try_start_debug_http_server(
    commands: &mut Commands,
    sim_log: &mut SimulationDebugLog,
    port: u16,
) {
    let (request_tx, request_rx) = mpsc::channel();
    let listen_addr = format!("127.0.0.1:{port}");
    let thread_tx = request_tx.clone();

    let thread = thread::spawn(move || super::standalone::run_http_thread(&listen_addr, thread_tx));

    sim_log.set_enabled(true);
    sim_log.log(
        0,
        format!("debug HTTP listening on http://127.0.0.1:{port}"),
    );
    eprintln!("OpenInfiniFactory debug HTTP: http://127.0.0.1:{port}");

    commands.insert_resource(DebugHttpBridge {
        receiver: Mutex::new(request_rx),
        _thread: thread,
        port,
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn process_pending_debug_http_start(
    mut commands: Commands,
    mut pending: ResMut<PendingDebugHttpStart>,
    mut sim_log: ResMut<SimulationDebugLog>,
    bridge: Option<Res<DebugHttpBridge>>,
) {
    if !pending.0 || bridge.is_some() {
        pending.0 = false;
        return;
    }
    pending.0 = false;
    try_start_debug_http_server(&mut commands, &mut sim_log, DEFAULT_DEBUG_HTTP_PORT);
}

pub fn poll_debug_http(
    mode: Res<State<GameMode>>,
    builder_mode: Res<BuilderMode>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    placement: Res<PlacementState>,
    perf_snapshot: DebugHttpPerfSnapshot,
    mut simulation: ResMut<SimulationState>,
    mut sim_log: ResMut<SimulationDebugLog>,
    mut presentation: ResMut<SimulationPresentationState>,
    mut pending_generated: ResMut<PendingGeneratedMaterials>,
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut turn_cache: ResMut<TurnCache>,
    worker: Option<Res<SimulationWorker>>,
    bridge: Option<Res<DebugHttpBridge>>,
    mut edit_history: ResMut<EditHistory>,
    mut playing: PlayingWorldParams,
) {
    let Some(bridge) = bridge else {
        return;
    };
    let render_ready = playing.render_assets.is_some();
    while let Ok(request) = bridge.receiver.lock().unwrap().try_recv() {
        let body = handle_embedded_debug_command(
            request.command,
            *mode.get(),
            *builder_mode,
            &playing_ui,
            &ui_runtime,
            &placement,
            &perf_snapshot,
            &mut simulation,
            &mut sim_log,
            &mut presentation,
            &mut pending_generated,
            &mut signal_cache,
            &mut turn_cache,
            worker.as_deref(),
            render_ready,
            &mut playing,
            &mut edit_history,
        );
        let _ = request.respond_to.send(body);
    }
}

fn handle_embedded_debug_command(
    command: DebugHttpCommand,
    mode: GameMode,
    builder_mode: BuilderMode,
    playing_ui: &PlayingUiState,
    ui_runtime: &UiRuntime,
    placement: &PlacementState,
    perf_snapshot: &DebugHttpPerfSnapshot<'_, '_>,
    simulation: &mut SimulationState,
    sim_log: &mut SimulationDebugLog,
    presentation: &mut SimulationPresentationState,
    pending_generated: &mut PendingGeneratedMaterials,
    signal_cache: &mut SignalNetworkCache,
    turn_cache: &mut TurnCache,
    worker: Option<&SimulationWorker>,
    render_ready: bool,
    playing: &mut PlayingWorldParams,
    edit_history: &mut EditHistory,
) -> String {
    if matches!(command, DebugHttpCommand::GetPerf) {
        return json_ok(perf_snapshot.capture(
            builder_mode,
            simulation,
            playing.world.blocks.len(),
        ));
    }

    if mode != GameMode::Playing {
        return json_error("game is not in Playing mode");
    }

    match command {
        DebugHttpCommand::GetPerf => unreachable!(),
        DebugHttpCommand::Help => help_json(),
        DebugHttpCommand::BlockKinds => block_kinds_json(),
        DebugHttpCommand::GetPosBlock { x, y, z } => {
            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                let pos = IVec3::new(x, y, z);
                serde_json::json!({
                    "ok": true,
                    "pos": pos_json(pos),
                    "block": block_json(&playing.world, pos),
                    "cursor": cursor_target_json(placement, &playing.world),
                })
                .to_string()
            } else {
                serde_json::json!({
                    "ok": true,
                    "cursor": cursor_target_json(placement, &playing.world),
                })
                .to_string()
            }
        }
        DebugHttpCommand::GetStatus => serde_json::json!({
            "ok": true,
            "simulation": simulation_status_json(simulation, builder_mode),
            "cursor": cursor_target_json(placement, &playing.world),
            "render_ready": render_ready,
            "active_play": playing_ui.active_play(),
            "ui_blocks_gameplay": ui_runtime.blocks_gameplay(),
        })
        .to_string(),
        DebugHttpCommand::PlaceBlock {
            x,
            y,
            z,
            kind,
            facing,
        } => {
            let Some(kind) = parse_block_kind(&kind) else {
                return json_error(&format!("unknown block kind `{kind}`"));
            };
            let Some(facing) = parse_facing(&facing) else {
                return json_error(&format!("unknown facing `{facing}`"));
            };
            let pos = IVec3::new(x, y, z);
            match place_block(&mut playing.world, pos, kind, facing) {
                Ok(()) => {
                    refresh_world_after_edit(playing, pos);
                    // 模拟进行中时同步 committed/worker，否则下一回合快照会抹掉放置并留下幽灵实体
                    if simulation.is_active() {
                        presentation.committed_world = playing.world.clone();
                        turn_cache.reset_to_turn(simulation.turn);
                        if let Some(worker) = worker {
                            worker.reset(
                                SimSnapshot::from_world(
                                    &playing.world,
                                    pending_generated,
                                    signal_cache,
                                    &playing.structure_state,
                                    &playing.movement_influence,
                                    &playing.pusher_state,
                                ),
                                simulation.turn,
                            );
                        }
                    }
                    json_ok(serde_json::json!({
                        "pos": pos_json(pos),
                        "block": block_json(&playing.world, pos),
                    }))
                }
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::Run => {
            if builder_mode != BuilderMode::Play {
                return json_error("switch to Play mode first");
            }
            if !playing_ui.active_play() || ui_runtime.blocks_gameplay() {
                return json_error("gameplay UI is blocking simulation controls");
            }
            if !render_ready {
                return json_error("world render assets are not ready");
            }
            let starting = !simulation.is_active();
            start_simulation_if_needed(
                simulation,
                &playing.world,
                &mut playing.structure_state,
                &mut playing.pusher_state,
                edit_history,
            );
            if starting {
                presentation.committed_world = playing.world.clone();
                presentation.last_render_powered_wires.clear();
                turn_cache.reset_to_turn(simulation.turn);
                if let Some(worker) = worker {
                    worker.reset(
                        SimSnapshot::from_world(
                            &playing.world,
                            pending_generated,
                            signal_cache,
                            &playing.structure_state,
                            &playing.movement_influence,
                            &playing.pusher_state,
                        ),
                        simulation.turn,
                    );
                }
            }
            request_continuous_run(simulation);
            sim_log.log(simulation.turn, "HTTP /run");
            serde_json::json!({
                "ok": true,
                "simulation": simulation_status_json(simulation, builder_mode),
            })
            .to_string()
        }
        DebugHttpCommand::RunOneTurn => {
            if builder_mode != BuilderMode::Play {
                return json_error("switch to Play mode first");
            }
            if !playing_ui.active_play() || ui_runtime.blocks_gameplay() {
                return json_error("gameplay UI is blocking simulation controls");
            }
            if !render_ready {
                return json_error("world render assets are not ready");
            }
            let starting = !simulation.is_active();
            start_simulation_if_needed(
                simulation,
                &playing.world,
                &mut playing.structure_state,
                &mut playing.pusher_state,
                edit_history,
            );
            if starting {
                presentation.committed_world = playing.world.clone();
                presentation.last_render_powered_wires.clear();
                turn_cache.reset_to_turn(simulation.turn);
                if let Some(worker) = worker {
                    worker.reset(
                        SimSnapshot::from_world(
                            &playing.world,
                            pending_generated,
                            signal_cache,
                            &playing.structure_state,
                            &playing.movement_influence,
                            &playing.pusher_state,
                        ),
                        simulation.turn,
                    );
                }
                // is_active = running || turn>0；先拉起 running 才能单步
                request_continuous_run(simulation);
            }
            match request_one_turn(simulation) {
                Ok(()) => {
                    sim_log.log(simulation.turn.saturating_add(1), "HTTP /runOneTurn queued");
                    serde_json::json!({
                        "ok": true,
                        "simulation": simulation_status_json(simulation, builder_mode),
                    })
                    .to_string()
                }
                Err(error) => json_error(error),
            }
        }
        DebugHttpCommand::GetLogs { limit } => sim_log.recent_json(limit),
        DebugHttpCommand::ClearLogs => {
            sim_log.clear();
            r#"{"ok":true}"#.into()
        }
        DebugHttpCommand::RunN { .. }
        | DebugHttpCommand::WorldReset
        | DebugHttpCommand::BeginSimulation
        | DebugHttpCommand::LoadSave { .. }
        | DebugHttpCommand::LoadFixture { .. }
        | DebugHttpCommand::RunFixture { .. }
        | DebugHttpCommand::RunAllFixtures => {
            json_error("use headless oif-debug-http binary for world/fixture API")
        }
    }
}
