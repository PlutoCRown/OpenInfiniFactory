use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::blocks::BlockData;
use crate::game::simulation::core::prepare_upcoming_generation;
use crate::game::state::{BuilderMode, SimulationState};
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{
    AnimationTiming, BlockAnimation, BlockAnimationKind, SIMULATION_TURN_SECONDS,
};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_pending_generated_previews, spawn_pending_generated_block, PendingGeneratedPreview,
    WorldRenderAssets,
};
use crate::scene::{apply_turn_output, BlockEntityIndex};
use crate::sim_core::{CachedTurn, SimSnapshot, SimulationWorker, TurnCache};

use super::movement::PusherState;
use super::structure_state::StructureState;
use super::structures::MovementInfluenceCache;

pub use super::behaviors::LaserBeam;
pub use super::signals::detector_is_active_public;
pub use super::signals::SignalNetworkCache;

#[derive(Resource, Clone)]
pub struct SimulationStepStats {
    pub has_sample: bool,
    pub total_ms: f64,
    pub prep_ms: f64,
    pub gravity_ms: f64,
    pub signal_ms: f64,
    pub marker_before_move_ms: f64,
    pub movement_mark_ms: f64,
    pub movement_execute_ms: f64,
    pub marker_after_move_ms: f64,
    pub behavior_ms: f64,
    pub signal_refresh_ms: f64,
    pub render_rebuild_ms: f64,
}

#[derive(Resource, Default, Clone)]
pub struct PendingGeneratedMaterials {
    pending: HashMap<IVec3, PendingGeneratedMaterial>,
}

impl PendingGeneratedMaterials {
    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub(crate) fn pending_keys(&self) -> impl Iterator<Item = IVec3> + '_ {
        self.pending.keys().copied()
    }

    pub(crate) fn insert_pending(&mut self, pos: IVec3, block: BlockData, ready_turn: u64) {
        self.pending
            .entry(pos)
            .or_insert(PendingGeneratedMaterial { block, ready_turn });
    }

    pub(crate) fn ready_pending_positions(&self, turn: u64) -> Vec<IVec3> {
        self.pending
            .iter()
            .filter_map(|(pos, pending)| (pending.ready_turn <= turn).then_some(*pos))
            .collect()
    }

    pub(crate) fn take_pending_block(&mut self, pos: IVec3) -> Option<BlockData> {
        self.pending.remove(&pos).map(|pending| pending.block)
    }

    pub(crate) fn pending_entries(&self) -> impl Iterator<Item = (IVec3, BlockData, u64)> + '_ {
        self.pending
            .iter()
            .map(|(pos, pending)| (*pos, pending.block, pending.ready_turn))
    }
}

#[derive(Clone)]
struct PendingGeneratedMaterial {
    block: BlockData,
    ready_turn: u64,
}

impl Default for SimulationStepStats {
    fn default() -> Self {
        Self {
            has_sample: false,
            total_ms: 0.0,
            prep_ms: 0.0,
            gravity_ms: 0.0,
            signal_ms: 0.0,
            marker_before_move_ms: 0.0,
            movement_mark_ms: 0.0,
            movement_execute_ms: 0.0,
            marker_after_move_ms: 0.0,
            behavior_ms: 0.0,
            signal_refresh_ms: 0.0,
            render_rebuild_ms: 0.0,
        }
    }
}

#[derive(Default)]
pub struct SimulationPresentationState {
    pub committed_world: WorldBlocks,
    pub last_render_powered_wires: HashSet<IVec3>,
}

pub fn apply_sim_snapshot(
    snapshot: &SimSnapshot,
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    signal_cache: &mut SignalNetworkCache,
    structure_state: &mut StructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
) {
    *world = snapshot.world.clone();
    *pending_generated = snapshot.pending_generated.clone();
    *signal_cache = snapshot.signal_cache.clone();
    *structure_state = snapshot.structure_state.clone();
    *movement_influence = snapshot.movement_influence.clone();
    *pusher_state = snapshot.pusher_state.clone();
}

pub fn poll_simulation_worker(
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    worker: Option<Res<SimulationWorker>>,
    mut turn_cache: ResMut<TurnCache>,
) {
    let Some(worker) = worker else {
        return;
    };
    if *builder_mode != BuilderMode::Play {
        return;
    }
    turn_cache.ingest_worker_results(worker.drain_results());
    worker.configure(
        simulation.turn,
        simulation.running,
        simulation.step_requested,
        simulation.speed,
        simulation.is_active(),
    );
}

pub fn prefetch_simulation_turn(
    _builder_mode: Res<BuilderMode>,
    _simulation: Res<SimulationState>,
    _world: ResMut<WorldBlocks>,
    _pending_generated: ResMut<PendingGeneratedMaterials>,
    _signal_cache: ResMut<SignalNetworkCache>,
    _turn_cache: ResMut<TurnCache>,
) {
}

#[derive(SystemParam)]
pub struct SimulationTickDeps<'w> {
    world: ResMut<'w, WorldBlocks>,
    pending_generated: ResMut<'w, PendingGeneratedMaterials>,
    signal_cache: ResMut<'w, SignalNetworkCache>,
    structure_state: ResMut<'w, StructureState>,
    movement_influence: ResMut<'w, MovementInfluenceCache>,
    pusher_state: ResMut<'w, PusherState>,
    turn_cache: ResMut<'w, TurnCache>,
    sim_stats: ResMut<'w, SimulationStepStats>,
    presentation: ResMut<'w, SimulationPresentationState>,
    block_index: ResMut<'w, BlockEntityIndex>,
    meshes: ResMut<'w, Assets<Mesh>>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
    debug: Res<'w, DebugState>,
}

pub fn tick_simulation(
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut commands: Commands,
    pending_previews: Query<Entity, With<PendingGeneratedPreview>>,
    mut deps: SimulationTickDeps,
) {
    let Some(render_assets) = deps.render_assets.as_ref() else {
        return;
    };
    if *builder_mode != BuilderMode::Play || (!simulation.running && !simulation.step_requested) {
        prepare_upcoming_generation(
            &deps.world,
            &mut deps.pending_generated,
            simulation.turn + 1,
        );
        refresh_pending_generated_previews(
            &mut commands,
            &mut deps.meshes,
            &pending_previews,
            render_assets,
            &deps.world,
            &deps.pending_generated,
            simulation.turn,
            simulation.accumulator,
        );
        return;
    }

    let animation_duration_for = |running: bool, speed: f32| {
        if running {
            SIMULATION_TURN_SECONDS / speed.max(0.001)
        } else {
            SIMULATION_TURN_SECONDS
        }
    };

    if simulation.step_requested {
        if let Some(cached) = deps.turn_cache.take_pending(simulation.turn + 1) {
            simulation.step_requested = false;
            simulation.accumulator = 0.0;
            simulation.turn += 1;
            present_cached_turn(
                cached,
                animation_duration_for(simulation.running, simulation.speed),
                &mut deps.presentation,
                &mut deps.world,
                &mut deps.pending_generated,
                &mut deps.signal_cache,
                &mut deps.structure_state,
                &mut deps.movement_influence,
                &mut deps.pusher_state,
                &mut commands,
                &mut deps.meshes,
                &mut deps.block_index,
                render_assets,
                &deps.debug,
                &mut deps.sim_stats,
            );
        }
        prepare_upcoming_generation(
            &deps.world,
            &mut deps.pending_generated,
            simulation.turn + 1,
        );
        refresh_pending_generated_previews(
            &mut commands,
            &mut deps.meshes,
            &pending_previews,
            render_assets,
            &deps.world,
            &deps.pending_generated,
            simulation.turn,
            simulation.accumulator,
        );
        return;
    }

    simulation.accumulator += time.delta_secs() * simulation.speed / SIMULATION_TURN_SECONDS;
    while simulation.accumulator >= 1.0 {
        let Some(cached) = deps.turn_cache.take_pending(simulation.turn + 1) else {
            break;
        };
        simulation.turn += 1;
        simulation.accumulator -= 1.0;
        present_cached_turn(
            cached,
            animation_duration_for(simulation.running, simulation.speed),
            &mut deps.presentation,
            &mut deps.world,
            &mut deps.pending_generated,
            &mut deps.signal_cache,
            &mut deps.structure_state,
            &mut deps.movement_influence,
            &mut deps.pusher_state,
            &mut commands,
            &mut deps.meshes,
            &mut deps.block_index,
            render_assets,
            &deps.debug,
            &mut deps.sim_stats,
        );
    }

    prepare_upcoming_generation(
        &deps.world,
        &mut deps.pending_generated,
        simulation.turn + 1,
    );
    refresh_pending_generated_previews(
        &mut commands,
        &mut deps.meshes,
        &pending_previews,
        render_assets,
        &deps.world,
        &deps.pending_generated,
        simulation.turn,
        simulation.accumulator,
    );
}

pub fn present_cached_turn(
    cached: CachedTurn,
    animation_duration: f32,
    presentation: &mut SimulationPresentationState,
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    signal_cache: &mut SignalNetworkCache,
    structure_state: &mut StructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_index: &mut BlockEntityIndex,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    sim_stats: &mut SimulationStepStats,
) {
    let before = presentation.committed_world.clone();
    apply_sim_snapshot(
        &cached.after,
        world,
        pending_generated,
        signal_cache,
        structure_state,
        movement_influence,
        pusher_state,
    );
    *sim_stats = cached.output.stats.clone();
    apply_turn_output(
        &before,
        world,
        &cached.output,
        &presentation.last_render_powered_wires,
        animation_duration,
        commands,
        meshes,
        block_index,
        render_assets,
        debug,
        structure_state,
        sim_stats,
    );
    presentation.last_render_powered_wires = cached.output.render_powered_wires.clone();
    presentation.committed_world = world.clone();
}

fn refresh_pending_generated_previews(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pending_previews: &Query<Entity, With<PendingGeneratedPreview>>,
    render_assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pending_generated: &PendingGeneratedMaterials,
    turn: u64,
    accumulator: f32,
) {
    despawn_pending_generated_previews(commands, pending_previews);
    for (pos, block, ready_turn) in pending_generated.pending_entries() {
        let progress = if ready_turn <= turn {
            1.0
        } else if ready_turn == turn + 1 {
            accumulator
        } else {
            0.0
        }
        .clamp(0.0, 1.0);

        spawn_pending_generated_block(
            commands,
            meshes,
            render_assets,
            world,
            pos,
            block,
            Some(BlockAnimation {
                from_pos: pos,
                to_pos: pos,
                from_facing: block.facing,
                to_facing: block.facing,
                kind: BlockAnimationKind::SpawnScale,
                duration: Some(SIMULATION_TURN_SECONDS),
                progress: Some(progress),
            }),
            AnimationTiming::simulation(SIMULATION_TURN_SECONDS),
        );
    }
}
