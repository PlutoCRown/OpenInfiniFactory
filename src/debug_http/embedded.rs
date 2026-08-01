use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::sync::{Mutex, mpsc};
#[cfg(not(target_arch = "wasm32"))]
use std::thread::{self, JoinHandle};

use crate::debug_http::protocol::{
    DebugHttpCommand, DebugHttpRequest, help_json, json_error, json_ok,
};
use crate::debug_http::snapshot::{
    acceptors_json, block_json, block_json_with_structure, cursor_target_json, embedded_status_json,
    perf_stats_json, player_entry_json, pos_json, power_query_json, resolve_pos_query,
    resolve_structure_query, simulation_status_json,
};
use crate::debug_http::world_ops::{block_kinds_json, parse_block_kind, parse_facing, place_block};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::debug::SimulationDebugLog;
use crate::game::player::controller::{FlyCamera, apply_player_save};
use crate::game::session::{self, PlayingWorldParams};
use crate::game::simulation::pending::PendingGeneratedMaterials;
use crate::game::simulation::signals::SignalNetworkCache;
use crate::game::simulation::stats::SimulationStepStats;
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    WorldEntryMode,
};
use crate::game::systems::perf::PerfStats;
use crate::game::systems::simulation_controls::{
    request_continuous_run, request_one_turn, start_simulation_if_needed,
};
use crate::game::ui::UiRuntime;
use crate::game::world::animation::{AnimatedBlock, AnimatedPusherRod};
use crate::game::world::rendering::BlockEntity;
use crate::shared::launch::{DEFAULT_DEBUG_HTTP_PORT, LaunchOptions};
use crate::shared::save::{SaveKind, SaveSlot, SaveState};
use crate::sim_bridge::SimulationPresentationState;
use crate::sim_bridge::{SimSnapshot, SimulationWorker, TurnCache};

#[derive(Resource)]
pub struct DebugHttpBridge {
    receiver: Mutex<mpsc::Receiver<DebugHttpRequest>>,
    #[cfg(not(target_arch = "wasm32"))]
    _thread: JoinHandle<()>,
    pub port: u16,
}

#[derive(Resource, Default)]
pub struct PendingDebugHttpStart(pub bool);

/// 会话状态（合并参数，避免 poll 系统超限）
#[derive(SystemParam)]
pub struct DebugHttpSessionSnapshot<'w, 's> {
    mode: Res<'w, State<GameMode>>,
    builder_mode: Res<'w, BuilderMode>,
    playing_ui: Res<'w, PlayingUiState>,
    ui_runtime: Res<'w, UiRuntime>,
    save_state: Res<'w, SaveState>,
    solution_state: Res<'w, SolutionState>,
    animated: Query<'w, 's, Entity, Or<(With<AnimatedBlock>, With<AnimatedPusherRod>)>>,
}

impl<'w, 's> DebugHttpSessionSnapshot<'w, 's> {
    fn animating(&self) -> bool {
        !self.animated.is_empty()
    }

    fn status_json(
        &self,
        simulation: &SimulationState,
        render_ready: bool,
        cursor: serde_json::Value,
    ) -> serde_json::Value {
        embedded_status_json(
            *self.mode.get(),
            *self.builder_mode,
            &self.playing_ui,
            &self.ui_runtime,
            simulation,
            &self.save_state,
            &self.solution_state,
            render_ready,
            self.animating(),
            cursor,
        )
    }
}

/// 供 HTTP /perf 读取的帧统计（合并参数，避免 poll 系统超限）
#[derive(SystemParam)]
pub struct DebugHttpPerfSnapshot<'w, 's> {
    perf: Res<'w, PerfStats>,
    diagnostics: Res<'w, DiagnosticsStore>,
    sim_stats: Res<'w, SimulationStepStats>,
    block_entities: Query<'w, 's, Entity, With<BlockEntity>>,
}

impl<'w, 's> DebugHttpPerfSnapshot<'w, 's> {
    fn capture(
        &self,
        builder_mode: BuilderMode,
        simulation: &SimulationState,
        block_count: usize,
        player_pos: Option<Vec3>,
    ) -> serde_json::Value {
        let fps = self
            .diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
            .unwrap_or(0.0);
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
    mut commands: Commands,
    session: DebugHttpSessionSnapshot,
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
    mut playing: PlayingWorldParams,
    mut player: Query<(&mut Transform, &mut FlyCamera), With<FlyCamera>>,
) {
    let Some(bridge) = bridge else {
        return;
    };
    let render_ready = playing.render_assets.is_some();
    while let Ok(request) = bridge.receiver.lock().unwrap().try_recv() {
        let body = handle_embedded_debug_command(
            request.command,
            &mut commands,
            &session,
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
            &mut player,
        );
        let _ = request.respond_to.send(body);
    }
}

fn handle_embedded_debug_command(
    command: DebugHttpCommand,
    commands: &mut Commands,
    session: &DebugHttpSessionSnapshot<'_, '_>,
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
    player: &mut Query<'_, '_, (&mut Transform, &mut FlyCamera), With<FlyCamera>>,
) -> String {
    let mode = *session.mode.get();
    let builder_mode = *session.builder_mode;
    let playing_ui = &*session.playing_ui;
    let ui_runtime = &*session.ui_runtime;
    let animating = session.animating();

    match &command {
        DebugHttpCommand::Help => return help_json(),
        DebugHttpCommand::BlockKinds => return block_kinds_json(),
        DebugHttpCommand::GetPerf => {
            let player_pos = player
                .single()
                .ok()
                .map(|(transform, _)| transform.translation);
            return json_ok(perf_snapshot.capture(
                builder_mode,
                simulation,
                playing.world.blocks.len(),
                player_pos,
            ));
        }
        DebugHttpCommand::GetStatus => {
            let cursor = if mode == GameMode::Playing {
                cursor_target_json(placement, &playing.world)
            } else {
                serde_json::Value::Null
            };
            return session
                .status_json(
                    simulation,
                    render_ready && mode == GameMode::Playing,
                    cursor,
                )
                .to_string();
        }
        DebugHttpCommand::GetPlayers => {
            if mode != GameMode::Playing {
                return json_ok(serde_json::json!({ "players": [] }));
            }
            let look_target = cursor_target_json(placement, &playing.world);
            let players = match player.single() {
                Ok((transform, _)) => {
                    vec![player_entry_json(transform.translation, look_target)]
                }
                Err(_) => Vec::new(),
            };
            return json_ok(serde_json::json!({ "players": players }));
        }
        DebugHttpCommand::LoadSave { name } => {
            if name.is_empty() {
                return json_error("session/enter requires ?name=");
            }
            if mode == GameMode::Playing {
                return json_error("already in world; exit first");
            }
            if mode != GameMode::StartMenu {
                return json_error("session/enter only from start menu");
            }
            let Some(slot) = SaveSlot::from_storage_path(name) else {
                return json_error(&format!("invalid save path `{name}`"));
            };
            let entry = match slot.kind() {
                SaveKind::Puzzle => WorldEntryMode::EditPuzzle,
                SaveKind::Solution => WorldEntryMode::PlaySolution,
                SaveKind::Free => WorldEntryMode::Free,
            };
            session::load_world(commands, slot, entry);
            return json_ok(serde_json::json!({
                "queued": true,
                "save": name,
                "entry": match entry {
                    WorldEntryMode::EditPuzzle => "edit_puzzle",
                    WorldEntryMode::PlaySolution => "play_solution",
                    WorldEntryMode::Free => "free",
                },
            }));
        }
        DebugHttpCommand::SessionExit => {
            if mode != GameMode::Playing {
                return json_error("not in world");
            }
            session::exit_to_main_menu(commands, false);
            return json_ok(serde_json::json!({ "exited": true }));
        }
        DebugHttpCommand::SessionSave => {
            if mode != GameMode::Playing {
                return json_error("not in world");
            }
            session::save_current_world(commands);
            return json_ok(serde_json::json!({ "queued": true }));
        }
        _ => {}
    }

    if mode != GameMode::Playing {
        return json_error("game is not in Playing mode");
    }

    match command {
        DebugHttpCommand::Help
        | DebugHttpCommand::BlockKinds
        | DebugHttpCommand::GetPerf
        | DebugHttpCommand::GetStatus
        | DebugHttpCommand::GetPlayers
        | DebugHttpCommand::LoadSave { .. }
        | DebugHttpCommand::SessionExit
        | DebugHttpCommand::SessionSave => unreachable!(),
        DebugHttpCommand::GetPosBlock {
            x,
            y,
            z,
            block_id,
        } => match resolve_pos_query(&playing.world, x, y, z, block_id) {
            Ok(pos) => serde_json::json!({
                "ok": true,
                "pos": pos_json(pos),
                "block": block_json_with_structure(
                    &playing.world,
                    Some(&playing.structure_state),
                    pos,
                ),
                "cursor": cursor_target_json(placement, &playing.world),
            })
            .to_string(),
            Err(error) => {
                if x.is_none() && y.is_none() && z.is_none() && block_id.is_none() {
                    serde_json::json!({
                        "ok": true,
                        "cursor": cursor_target_json(placement, &playing.world),
                    })
                    .to_string()
                } else {
                    json_error(&error)
                }
            }
        },
        DebugHttpCommand::GetStructure {
            x,
            y,
            z,
            block_id,
            structure_id,
        } => match resolve_structure_query(
            &playing.world,
            &mut playing.structure_state,
            x,
            y,
            z,
            block_id,
            structure_id,
        ) {
            Ok(structure) => json_ok(serde_json::json!({ "structure": structure })),
            Err(error) => json_error(&error),
        },
        DebugHttpCommand::GetPower {
            x,
            y,
            z,
            block_id,
        } => match resolve_pos_query(&playing.world, x, y, z, block_id) {
            Ok(pos) => json_ok(power_query_json(
                &mut signal_cache.0,
                &playing.world,
                pos,
            )),
            Err(error) => json_error(&error),
        },
        DebugHttpCommand::GetAcceptors => json_ok(serde_json::json!({
            "acceptors": acceptors_json(&playing.world, &playing.structure_state),
        })),
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
            );
            if starting {
                presentation.committed_world = playing.world.clone();
                presentation.last_powered_wires.clear();
                simulation.last_powered_devices.clear();
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
                "simulation": simulation_status_json(simulation, builder_mode, animating),
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
            );
            if starting {
                presentation.committed_world = playing.world.clone();
                presentation.last_powered_wires.clear();
                simulation.last_powered_devices.clear();
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
                request_continuous_run(simulation);
            }
            match request_one_turn(simulation) {
                Ok(()) => {
                    sim_log.log(simulation.turn.saturating_add(1), "HTTP /runOneTurn queued");
                    serde_json::json!({
                        "ok": true,
                        "simulation": simulation_status_json(simulation, builder_mode, animating),
                    })
                    .to_string()
                }
                Err(error) => json_error(error),
            }
        }
        DebugHttpCommand::BeginSimulation => {
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
            );
            if starting {
                presentation.committed_world = playing.world.clone();
                presentation.last_powered_wires.clear();
                simulation.last_powered_devices.clear();
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
            sim_log.log(simulation.turn, "HTTP /beginSimulation");
            serde_json::json!({
                "ok": true,
                "simulation": simulation_status_json(simulation, builder_mode, animating),
                "started": starting,
            })
            .to_string()
        }
        DebugHttpCommand::SimPause => {
            simulation.running = false;
            json_ok(serde_json::json!({
                "simulation": simulation_status_json(simulation, builder_mode, animating),
            }))
        }
        DebugHttpCommand::RunN { n } => {
            if builder_mode != BuilderMode::Play {
                return json_error("switch to Play mode first");
            }
            if !playing_ui.active_play() || ui_runtime.blocks_gameplay() {
                return json_error("gameplay UI is blocking simulation controls");
            }
            if !render_ready {
                return json_error("world render assets are not ready");
            }
            // 嵌入式：排队 n 次单步意图不现实；提示用无头或连续 /run
            let _ = n;
            json_error("use headless oif-debug-http for /sim/run?n=; embedded supports /run and /runOneTurn")
        }
        DebugHttpCommand::GetLogs { limit } => sim_log.recent_json(limit),
        DebugHttpCommand::ClearLogs => {
            sim_log.clear();
            r#"{"ok":true}"#.into()
        }
        DebugHttpCommand::WorldReset => {
            json_error("use headless oif-debug-http binary for world/reset")
        }
        DebugHttpCommand::TeleportPlayer {
            x,
            y,
            z,
            yaw,
            pitch,
            look_at,
        } => {
            let Ok((mut transform, mut camera)) = player.single_mut() else {
                return json_error("player entity not found");
            };
            let mut save = crate::game::player::controller::capture_player_save(&camera, &transform);
            save.x = x;
            save.y = y;
            save.z = z;
            if let Some((lx, ly, lz)) = look_at {
                let eye = Vec3::new(x, y, z);
                // lookAt 按格子坐标，看向方块中心
                let target = Vec3::new(lx, ly, lz) + Vec3::splat(0.5);
                let dir = (target - eye).normalize_or_zero();
                if dir != Vec3::ZERO {
                    save.yaw = (-dir.x).atan2(-dir.z);
                    save.pitch = dir.y.asin().clamp(-1.54, 1.54);
                }
            } else {
                if let Some(yaw) = yaw {
                    save.yaw = yaw;
                }
                if let Some(pitch) = pitch {
                    save.pitch = pitch;
                }
            }
            apply_player_save(&mut camera, &mut transform, &save);
            json_ok(serde_json::json!({
                "position": { "x": save.x, "y": save.y, "z": save.z },
                "yaw": save.yaw,
                "pitch": save.pitch,
            }))
        }
    }
}
