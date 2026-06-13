use bevy::prelude::*;
use std::sync::{mpsc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::thread::{self, JoinHandle};

use crate::debug_http::protocol::{json_error, help_json, DebugHttpCommand, DebugHttpRequest};
use crate::debug_http::snapshot::{
    block_json, cursor_target_json, simulation_status_json,
};
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::structure_state::StructureState;
use crate::game::state::{BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState};
use crate::game::systems::simulation_controls::{
    request_continuous_run, request_one_turn, start_simulation_if_needed,
};
use crate::game::ui::UiRuntime;
use crate::game::world::grid::WorldBlocks;
use crate::shared::launch::LaunchOptions;
use crate::sim_core::SimulationDebugLog;

#[derive(Resource)]
pub struct DebugHttpBridge {
    receiver: Mutex<mpsc::Receiver<DebugHttpRequest>>,
    #[cfg(not(target_arch = "wasm32"))]
    _thread: JoinHandle<()>,
    pub port: u16,
}

pub struct DebugToolsPlugin;

impl Plugin for DebugToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationDebugLog>();

        #[cfg(not(target_arch = "wasm32"))]
        if app
            .world()
            .get_resource::<LaunchOptions>()
            .is_some_and(LaunchOptions::debug_http_enabled)
        {
            app.add_systems(Startup, start_debug_http_server);
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
) {
    let Some(port) = launch.debug_http_port else {
        return;
    };

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

pub fn poll_debug_http(
    mode: Res<State<GameMode>>,
    builder_mode: Res<BuilderMode>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    mut simulation: ResMut<SimulationState>,
    mut structure_state: ResMut<StructureState>,
    mut pusher_state: ResMut<PusherState>,
    mut sim_log: ResMut<SimulationDebugLog>,
    bridge: Option<Res<DebugHttpBridge>>,
    render_assets: Option<Res<crate::game::world::rendering::WorldRenderAssets>>,
) {
    let Some(bridge) = bridge else {
        return;
    };
    drain_debug_http_requests(
        &bridge,
        *mode.get(),
        *builder_mode,
        &playing_ui,
        &ui_runtime,
        &placement,
        &world,
        &mut simulation,
        &mut structure_state,
        &mut pusher_state,
        &mut sim_log,
        render_assets.is_some(),
    );
}

pub fn drain_debug_http_requests(
    bridge: &DebugHttpBridge,
    mode: GameMode,
    builder_mode: BuilderMode,
    playing_ui: &PlayingUiState,
    ui_runtime: &UiRuntime,
    placement: &PlacementState,
    world: &WorldBlocks,
    simulation: &mut SimulationState,
    structure_state: &mut StructureState,
    pusher_state: &mut PusherState,
    sim_log: &mut SimulationDebugLog,
    render_ready: bool,
) {
    while let Ok(request) = bridge.receiver.lock().unwrap().try_recv() {
        let body = handle_embedded_debug_command(
            request.command,
            mode,
            builder_mode,
            playing_ui,
            ui_runtime,
            placement,
            world,
            simulation,
            structure_state,
            pusher_state,
            sim_log,
            render_ready,
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
    world: &WorldBlocks,
    simulation: &mut SimulationState,
    structure_state: &mut StructureState,
    pusher_state: &mut PusherState,
    sim_log: &mut SimulationDebugLog,
    render_ready: bool,
) -> String {
    if mode != GameMode::Playing {
        return json_error("game is not in Playing mode");
    }

    match command {
        DebugHttpCommand::Help => help_json(),
        DebugHttpCommand::GetPosBlock { x, y, z } => {
            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                let pos = IVec3::new(x, y, z);
                serde_json::json!({
                    "ok": true,
                    "pos": crate::debug_http::snapshot::pos_json(pos),
                    "block": block_json(world, pos),
                    "cursor": cursor_target_json(placement, world),
                })
                .to_string()
            } else {
                serde_json::json!({
                    "ok": true,
                    "cursor": cursor_target_json(placement, world),
                })
                .to_string()
            }
        }
        DebugHttpCommand::GetStatus => serde_json::json!({
            "ok": true,
            "simulation": simulation_status_json(simulation, builder_mode),
            "cursor": cursor_target_json(placement, world),
            "render_ready": render_ready,
            "active_play": playing_ui.active_play(),
            "ui_blocks_gameplay": ui_runtime.blocks_gameplay(),
        })
        .to_string(),
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
            start_simulation_if_needed(simulation, world, structure_state, pusher_state);
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
            start_simulation_if_needed(simulation, world, structure_state, pusher_state);
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
        | DebugHttpCommand::BlockKinds
        | DebugHttpCommand::WorldReset
        | DebugHttpCommand::BeginSimulation
        | DebugHttpCommand::LoadSave { .. }
        | DebugHttpCommand::PlaceBlock { .. }
        | DebugHttpCommand::LoadFixture { .. }
        | DebugHttpCommand::RunFixture { .. }
        | DebugHttpCommand::RunAllFixtures => {
            json_error("use headless oif-debug-http binary for world/fixture API")
        }
    }
}
