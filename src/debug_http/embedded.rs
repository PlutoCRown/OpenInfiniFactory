use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::sync::{mpsc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::thread::{self, JoinHandle};

use crate::debug_http::embedded_session::{build_runtime_snapshot, simulation_control_adapter};
use crate::debug_http::introspection::{
    get_factory_block_state_json, get_factory_id_at_json, get_factory_pos_json,
    get_power_network_json, get_power_networks_json, get_powered_devices_json, get_region_json,
    get_structure_at_json, preview_movement_plan_json,
};
use crate::debug_http::protocol::{
    help_json, json_error, json_ok, DebugHttpCommand, DebugHttpRequest,
};
use crate::debug_http::snapshot::{block_json, cursor_target_json, simulation_status_json};
use crate::debug_http::world_layer::{
    parse_world_layer_option, resolve_world_blocks, world_layer_label, DebugWorldLayer,
};
use crate::game::simulation::core::simulate_turn;
use crate::game::simulation::movement::{pusher_head_position, PusherState};
use crate::game::simulation::runtime::present_cached_turn;
use crate::game::simulation::runtime::{
    PendingGeneratedMaterials, SignalNetworkCache, SimulationPresentationState, SimulationStepStats,
};
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::simulation::SimulationWorlds;
use crate::game::state::{BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState};
use crate::game::systems::debug::DebugState;
use crate::game::systems::simulation_controls::{
    request_continuous_run, start_simulation_if_needed,
};
use crate::game::ui::UiRuntime;
use crate::game::world::animation::SIMULATION_TURN_SECONDS;
use crate::game::world::block_instance::MaterialBlockRegistry;
use crate::game::world::factory_registry::{FactoryBlockId, FactoryBlockRegistry};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{BlockEntity, WorldRenderAssets};
use crate::shared::launch::LaunchOptions;
use crate::sim_core::{CachedTurn, SimSnapshot};
use crate::sim_core::{SimulationDebugLog, SimulationWorker, TurnCache};

#[derive(Resource)]
pub struct DebugHttpBridge {
    receiver: Mutex<mpsc::Receiver<DebugHttpRequest>>,
    #[cfg(not(target_arch = "wasm32"))]
    _thread: JoinHandle<()>,
    pub port: u16,
}

#[derive(SystemParam)]
pub struct EmbeddedSimDeps<'w, 's> {
    commands: Commands<'w, 's>,
    world: ResMut<'w, WorldBlocks>,
    simulation: ResMut<'w, SimulationState>,
    structure_state: ResMut<'w, StructureState>,
    factory_registry: ResMut<'w, FactoryBlockRegistry>,
    material_registry: ResMut<'w, MaterialBlockRegistry>,
    pusher_state: ResMut<'w, PusherState>,
    pending_generated: ResMut<'w, PendingGeneratedMaterials>,
    signal_cache: ResMut<'w, SignalNetworkCache>,
    movement_influence: ResMut<'w, MovementInfluenceCache>,
    presentation: ResMut<'w, SimulationPresentationState>,
    block_entities: Query<'w, 's, (Entity, &'static BlockEntity)>,
    meshes: ResMut<'w, Assets<Mesh>>,
    debug: Res<'w, DebugState>,
    sim_stats: ResMut<'w, SimulationStepStats>,
    turn_cache: ResMut<'w, TurnCache>,
    worker: Option<Res<'w, SimulationWorker>>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
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

impl<'w, 's> EmbeddedSimDeps<'w, 's> {
    fn begin_embedded_simulation(&mut self) -> Result<(), &'static str> {
        if self.render_assets.is_none() {
            return Err("world render assets are not ready");
        }
        start_simulation_if_needed(
            &mut self.simulation,
            &self.world,
            &mut self.structure_state,
            &mut self.factory_registry,
            &mut self.material_registry,
            &mut self.pusher_state,
        );
        self.presentation.committed_world = self.world.clone();
        self.presentation.last_render_powered_wires.clear();
        self.turn_cache.reset_to_turn(self.simulation.turn);
        if let Some(worker) = self.worker.as_ref() {
            worker.reset(
                SimSnapshot::at_simulation_start(
                    &self.world,
                    &self.pending_generated,
                    &self.signal_cache,
                    &self.movement_influence,
                    &self.pusher_state,
                ),
                self.simulation.turn,
            );
        }
        Ok(())
    }

    fn advance_embedded_turns(
        &mut self,
        n: u64,
        sim_log: &mut SimulationDebugLog,
    ) -> Result<(), &'static str> {
        if n == 0 {
            return Ok(());
        }
        self.begin_embedded_simulation()?;

        let Some(render_assets) = self.render_assets.as_ref() else {
            return Err("world render assets are not ready");
        };

        self.simulation.running = false;
        self.simulation.step_requested = false;
        self.simulation.speed = 1.0;

        let mut snapshot = build_runtime_snapshot(
            &self.simulation,
            &self.world,
            &self.structure_state,
            &self.factory_registry,
            &self.material_registry,
            &self.pending_generated,
            &self.signal_cache,
            &self.movement_influence,
            &self.pusher_state,
        )?;

        for _ in 0..n {
            let next_turn = self.simulation.turn + 1;
            let mut worlds = SimulationWorlds::from_snapshot_parts(
                snapshot.solution.clone(),
                snapshot.solution_structures.clone(),
                snapshot.world.clone(),
                snapshot.structure_state.clone(),
                snapshot.factory_registry.clone(),
                snapshot.material_registry.clone(),
            );
            let output = simulate_turn(
                &mut worlds,
                &mut snapshot.pending_generated,
                &mut snapshot.signal_cache,
                next_turn,
                &mut snapshot.pusher_state,
                &mut snapshot.movement_influence,
                Some(sim_log),
                Some(&mut self.sim_stats),
            );
            snapshot.world = worlds.turn;
            snapshot.structure_state = worlds.turn_structures;
            snapshot.factory_registry = worlds.factory_registry;
            snapshot.material_registry = worlds.material_registry;
            self.simulation.turn = next_turn;
            let cached = CachedTurn {
                output,
                after: snapshot.clone(),
            };
            present_cached_turn(
                cached,
                SIMULATION_TURN_SECONDS,
                &mut self.presentation,
                &mut self.world,
                &mut self.pending_generated,
                &mut self.signal_cache,
                &mut self.structure_state,
                &mut self.factory_registry,
                &mut self.material_registry,
                &mut self.movement_influence,
                &mut self.pusher_state,
                &mut self.commands,
                &mut self.meshes,
                &self.block_entities,
                render_assets,
                &self.debug,
                &mut self.sim_stats,
            );
        }

        self.turn_cache.reset_to_turn(self.simulation.turn);
        if let Some(worker) = self.worker.as_ref() {
            worker.reset(snapshot, self.simulation.turn);
        }

        Ok(())
    }
}

pub fn poll_debug_http(
    mode: Res<State<GameMode>>,
    builder_mode: Res<BuilderMode>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    placement: Res<PlacementState>,
    mut sim_log: ResMut<SimulationDebugLog>,
    bridge: Option<Res<DebugHttpBridge>>,
    mut sim_deps: EmbeddedSimDeps,
) {
    let Some(bridge) = bridge else {
        return;
    };
    let render_ready = sim_deps.render_assets.is_some();
    while let Ok(request) = bridge.receiver.lock().unwrap().try_recv() {
        let body = handle_embedded_debug_command(
            request.command,
            *mode.get(),
            *builder_mode,
            &playing_ui,
            &ui_runtime,
            &placement,
            &mut sim_log,
            render_ready,
            &mut sim_deps,
        );
        let _ = request.respond_to.send(body);
    }
}

fn embedded_http_sim_ready(
    builder_mode: BuilderMode,
    render_ready: bool,
) -> Result<(), &'static str> {
    if builder_mode != BuilderMode::Play {
        return Err("switch to Play mode first");
    }
    if !render_ready {
        return Err("world render assets are not ready");
    }
    Ok(())
}

fn handle_embedded_debug_command(
    command: DebugHttpCommand,
    mode: GameMode,
    builder_mode: BuilderMode,
    playing_ui: &PlayingUiState,
    ui_runtime: &UiRuntime,
    placement: &PlacementState,
    sim_log: &mut SimulationDebugLog,
    render_ready: bool,
    sim_deps: &mut EmbeddedSimDeps<'_, '_>,
) -> String {
    if mode != GameMode::Playing {
        return json_error("game is not in Playing mode");
    }

    match command {
        DebugHttpCommand::Help => help_json(),
        DebugHttpCommand::GetPosBlock { x, y, z, world } => {
            let layer = parse_world_layer_option(world.as_deref());
            let control = simulation_control_adapter(&sim_deps.simulation);
            let turn = &*sim_deps.world;
            let query_world = match resolve_world_blocks(layer, turn, &control) {
                Ok(world) => world,
                Err(error) => return json_error(&error),
            };
            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                let pos = IVec3::new(x, y, z);
                serde_json::json!({
                    "ok": true,
                    "world": world_layer_label(layer),
                    "pos": crate::debug_http::snapshot::pos_json(pos),
                    "block": block_json(query_world, pos),
                    "cursor": cursor_target_json(placement, turn),
                })
                .to_string()
            } else {
                serde_json::json!({
                    "ok": true,
                    "cursor": cursor_target_json(placement, turn),
                })
                .to_string()
            }
        }
        DebugHttpCommand::GetStatus => {
            let world = &*sim_deps.world;
            serde_json::json!({
                "ok": true,
                "simulation": simulation_status_json(&sim_deps.simulation, builder_mode),
                "cursor": cursor_target_json(placement, world),
                "render_ready": render_ready,
                "active_play": playing_ui.active_play(),
                "ui_blocks_gameplay": ui_runtime.blocks_gameplay(),
            })
            .to_string()
        }
        DebugHttpCommand::BeginSimulation => match sim_deps.begin_embedded_simulation() {
            Ok(()) => {
                sim_log.log(sim_deps.simulation.turn, "HTTP /beginSimulation");
                json_ok(serde_json::json!({
                    "simulation": simulation_status_json(&sim_deps.simulation, builder_mode),
                }))
            }
            Err(error) => json_error(error),
        },
        DebugHttpCommand::Run => {
            if let Err(error) = embedded_http_sim_ready(builder_mode, render_ready) {
                return json_error(error);
            }
            start_simulation_if_needed(
                &mut sim_deps.simulation,
                &sim_deps.world,
                &mut sim_deps.structure_state,
                &mut sim_deps.factory_registry,
                &mut sim_deps.material_registry,
                &mut sim_deps.pusher_state,
            );
            request_continuous_run(&mut sim_deps.simulation);
            sim_log.log(sim_deps.simulation.turn, "HTTP /run");
            json_ok(serde_json::json!({
                "simulation": simulation_status_json(&sim_deps.simulation, builder_mode),
            }))
        }
        DebugHttpCommand::RunOneTurn => {
            if let Err(error) = embedded_http_sim_ready(builder_mode, render_ready) {
                return json_error(error);
            }
            match sim_deps.advance_embedded_turns(1, sim_log) {
                Ok(()) => {
                    sim_log.log(sim_deps.simulation.turn, "HTTP /runOneTurn");
                    json_ok(serde_json::json!({
                        "simulation": simulation_status_json(&sim_deps.simulation, builder_mode),
                    }))
                }
                Err(error) => json_error(error),
            }
        }
        DebugHttpCommand::RunN { n } => {
            if let Err(error) = embedded_http_sim_ready(builder_mode, render_ready) {
                return json_error(error);
            }
            match sim_deps.advance_embedded_turns(n, sim_log) {
                Ok(()) => {
                    sim_log.log(sim_deps.simulation.turn, format!("HTTP /runN n={n}"));
                    json_ok(serde_json::json!({
                        "simulation": simulation_status_json(&sim_deps.simulation, builder_mode),
                        "turns": n,
                    }))
                }
                Err(error) => json_error(error),
            }
        }
        DebugHttpCommand::GetExtendedDevices => {
            let world = &*sim_deps.world;
            let devices: Vec<_> = sim_deps
                .pusher_state
                .extended_device_positions()
                .into_iter()
                .map(|pos| {
                    serde_json::json!({
                        "pos": crate::debug_http::snapshot::pos_json(pos),
                        "head": pusher_head_position(world, pos)
                            .map(|head| crate::debug_http::snapshot::pos_json(head)),
                    })
                })
                .collect();
            json_ok(serde_json::json!({
                "count": devices.len(),
                "devices": devices,
            }))
        }
        DebugHttpCommand::GetRegion {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
            world,
        } => {
            let layer = parse_world_layer_option(world.as_deref());
            let control = simulation_control_adapter(&sim_deps.simulation);
            let Some((min_x, min_y, min_z, max_x, max_y, max_z)) = min_x
                .zip(min_y)
                .zip(min_z)
                .zip(max_x)
                .zip(max_y)
                .zip(max_z)
                .map(|(((((a, b), c), d), e), f)| (a, b, c, d, e, f))
            else {
                return json_error("getRegion requires min_x/y/z and max_x/y/z");
            };
            let query_world = match resolve_world_blocks(layer, &sim_deps.world, &control) {
                Ok(world) => world,
                Err(error) => return json_error(&error),
            };
            let mut params = std::collections::HashMap::new();
            params.insert("min_x".into(), min_x.to_string());
            params.insert("min_y".into(), min_y.to_string());
            params.insert("min_z".into(), min_z.to_string());
            params.insert("max_x".into(), max_x.to_string());
            params.insert("max_y".into(), max_y.to_string());
            params.insert("max_z".into(), max_z.to_string());
            match get_region_json(query_world, &params) {
                Ok(mut data) => {
                    if let Some(object) = data.as_object_mut() {
                        object.insert("world".into(), world_layer_label(layer).into());
                    }
                    json_ok(data)
                }
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::GetPowerNetworks => {
            sim_deps.signal_cache.ensure_fresh(&sim_deps.world);
            json_ok(get_power_networks_json(
                &sim_deps.world,
                &mut sim_deps.signal_cache,
            ))
        }
        DebugHttpCommand::GetPowerNetwork { id } => {
            let Some(id) = id else {
                return json_error("getPowerNetwork requires ?id=");
            };
            sim_deps.signal_cache.ensure_fresh(&sim_deps.world);
            match get_power_network_json(&sim_deps.world, &mut sim_deps.signal_cache, id) {
                Ok(data) => json_ok(data),
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::GetPoweredDevices => {
            sim_deps.signal_cache.ensure_fresh(&sim_deps.world);
            json_ok(get_powered_devices_json(
                &sim_deps.world,
                &mut sim_deps.signal_cache,
            ))
        }
        DebugHttpCommand::GetFactoryBlockState { x, y, z, world } => {
            let layer = parse_world_layer_option(world.as_deref());
            let Some((x, y, z)) = x.zip(y).zip(z).map(|((a, b), c)| (a, b, c)) else {
                return json_error("getFactoryBlockState requires ?x=&y=&z=");
            };
            let control = simulation_control_adapter(&sim_deps.simulation);
            let solution_structures = sim_deps
                .simulation
                .start_structures
                .clone()
                .unwrap_or_else(|| sim_deps.structure_state.clone());
            let structure_snapshot = sim_deps.structure_state.clone();
            let query_world = match resolve_world_blocks(layer, &sim_deps.world, &control) {
                Ok(world) => world,
                Err(error) => return json_error(&error),
            };
            sim_deps.signal_cache.ensure_fresh(&sim_deps.world);
            match get_factory_block_state_json(
                IVec3::new(x, y, z),
                query_world,
                &sim_deps.world,
                &structure_snapshot,
                &solution_structures,
                &sim_deps.factory_registry,
                &control,
                &mut sim_deps.signal_cache,
                layer,
            ) {
                Ok(data) => json_ok(data),
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::GetFactoryIdAt { x, y, z, world } => {
            let layer = parse_world_layer_option(world.as_deref());
            let Some((x, y, z)) = x.zip(y).zip(z).map(|((a, b), c)| (a, b, c)) else {
                return json_error("getFactoryIdAt requires ?x=&y=&z=");
            };
            let control = simulation_control_adapter(&sim_deps.simulation);
            let query_world = match resolve_world_blocks(layer, &sim_deps.world, &control) {
                Ok(world) => world,
                Err(error) => return json_error(&error),
            };
            match get_factory_id_at_json(
                IVec3::new(x, y, z),
                query_world,
                &sim_deps.factory_registry,
                layer,
            ) {
                Ok(data) => json_ok(data),
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::GetFactoryPos { id, world } => {
            let layer = parse_world_layer_option(world.as_deref());
            let Some(id) = id else {
                return json_error("getFactoryPos requires ?id=");
            };
            match get_factory_pos_json(
                FactoryBlockId::from_u32(id),
                &sim_deps.factory_registry,
                layer,
            ) {
                Ok(data) => json_ok(data),
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::GetStructureAt { x, y, z, world } => {
            let layer = parse_world_layer_option(world.as_deref());
            let Some((x, y, z)) = x.zip(y).zip(z).map(|((a, b), c)| (a, b, c)) else {
                return json_error("getStructureAt requires ?x=&y=&z=");
            };
            let structures = match layer {
                DebugWorldLayer::Turn => sim_deps.structure_state.clone(),
                DebugWorldLayer::Solution => sim_deps
                    .simulation
                    .start_structures
                    .clone()
                    .unwrap_or_else(|| sim_deps.structure_state.clone()),
            };
            match get_structure_at_json(IVec3::new(x, y, z), &structures, layer) {
                Ok(data) => json_ok(data),
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::PreviewMovementPlan => {
            let control = simulation_control_adapter(&sim_deps.simulation);
            sim_deps.signal_cache.ensure_fresh(&sim_deps.world);
            json_ok(preview_movement_plan_json(
                &sim_deps.world,
                &sim_deps.structure_state,
                &sim_deps.factory_registry,
                &control,
                &mut sim_deps.signal_cache,
                &sim_deps.pusher_state,
                &mut sim_deps.movement_influence,
            ))
        }
        DebugHttpCommand::GetLogs { limit } => sim_log.recent_json(limit),
        DebugHttpCommand::ClearLogs => {
            sim_log.clear();
            r#"{"ok":true}"#.into()
        }
        DebugHttpCommand::BlockKinds
        | DebugHttpCommand::WorldReset
        | DebugHttpCommand::LoadSave { .. }
        | DebugHttpCommand::PlaceBlock { .. }
        | DebugHttpCommand::LoadFixture { .. }
        | DebugHttpCommand::RunFixture { .. }
        | DebugHttpCommand::RunAllFixtures => {
            json_error("use headless oif-debug-http binary for world/fixture authoring API")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_http::embedded_session::build_runtime_snapshot;

    #[test]
    fn build_runtime_snapshot_requires_active_simulation() {
        let simulation = SimulationState::default();
        let world = WorldBlocks::default();
        let structures = StructureState::default();
        let pending = PendingGeneratedMaterials::default();
        let signal = SignalNetworkCache::default();
        let influence = MovementInfluenceCache::default();
        let pusher = PusherState::default();
        assert!(build_runtime_snapshot(
            &simulation,
            &world,
            &structures,
            &FactoryBlockRegistry::default(),
            &MaterialBlockRegistry::default(),
            &pending,
            &signal,
            &influence,
            &pusher,
        )
        .is_err());
    }
}
