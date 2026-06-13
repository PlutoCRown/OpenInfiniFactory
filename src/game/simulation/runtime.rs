use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::blocks::BlockData;
use crate::game::simulation::core::{prepare_upcoming_generation, simulate_turn};
use crate::game::state::{BuilderMode, SimulationState};
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{
    AnimationTiming, BlockAnimation, BlockAnimationKind, SIMULATION_TURN_SECONDS,
};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_pending_generated_previews, spawn_pending_generated_block, BlockEntity,
    PendingGeneratedPreview, WorldRenderAssets,
};
use crate::scene::apply_turn_output;
use crate::sim_core::{SimulationDebugLog, TurnCache};

use super::movement::PusherState;
use super::structure_state::StructureState;
use super::structures::MovementInfluenceCache;

pub use super::behaviors::LaserBeam;
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

#[derive(Resource, Default)]
pub struct PendingGeneratedMaterials {
    pending: HashMap<IVec3, PendingGeneratedMaterial>,
    pending_destroyed: HashMap<IVec3, u64>,
    pending_acceptance_sparks: HashMap<IVec3, u64>,
    pending_teleports: HashMap<IVec3, u64>,
}

impl PendingGeneratedMaterials {
    pub fn clear(&mut self) {
        self.pending.clear();
        self.pending_destroyed.clear();
        self.pending_acceptance_sparks.clear();
        self.pending_teleports.clear();
    }

    pub(super) fn mark_destroyed(&mut self, pos: IVec3, ready_turn: u64) {
        self.pending_destroyed.entry(pos).or_insert(ready_turn);
    }

    pub(super) fn mark_teleport(&mut self, entrance: IVec3, ready_turn: u64) {
        self.pending_teleports.entry(entrance).or_insert(ready_turn);
    }

    pub(super) fn remove_pending_teleport(&mut self, entrance: IVec3) {
        self.pending_teleports.remove(&entrance);
    }

    pub(super) fn ready_teleport_entrances(&self, turn: u64) -> Vec<IVec3> {
        self.pending_teleports
            .iter()
            .filter_map(|(entrance, ready_turn)| (*ready_turn <= turn).then_some(*entrance))
            .collect()
    }

    pub(super) fn mark_acceptance_spark(&mut self, pos: IVec3, ready_turn: u64) {
        self.pending_acceptance_sparks
            .entry(pos)
            .or_insert(ready_turn);
    }

    pub(crate) fn pending_destroy_turn(&self, pos: IVec3) -> Option<u64> {
        self.pending_destroyed.get(&pos).copied()
    }

    pub(crate) fn pending_acceptance_spark_turn(&self, pos: IVec3) -> Option<u64> {
        self.pending_acceptance_sparks.get(&pos).copied()
    }

    pub(crate) fn has_pending_destruction(&self) -> bool {
        !self.pending_destroyed.is_empty()
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

    pub(crate) fn ready_destroyed_positions(&self, turn: u64) -> Vec<IVec3> {
        self.pending_destroyed
            .iter()
            .filter_map(|(pos, ready_turn)| (*ready_turn <= turn).then_some(*pos))
            .collect()
    }

    pub(crate) fn remove_destroyed(&mut self, pos: IVec3) {
        self.pending_destroyed.remove(&pos);
    }

    pub(crate) fn take_acceptance_spark(&mut self, pos: IVec3) -> Option<u64> {
        self.pending_acceptance_sparks.remove(&pos)
    }

    pub(crate) fn pending_entries(&self) -> impl Iterator<Item = (IVec3, BlockData, u64)> + '_ {
        self.pending
            .iter()
            .map(|(pos, pending)| (*pos, pending.block, pending.ready_turn))
    }
}

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

#[derive(SystemParam)]
pub(crate) struct SimulationTurnDeps<'w> {
    structure_state: ResMut<'w, StructureState>,
    movement_influence: ResMut<'w, MovementInfluenceCache>,
    pusher_state: ResMut<'w, PusherState>,
    sim_log: ResMut<'w, SimulationDebugLog>,
}

pub fn prefetch_simulation_turn(
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    mut world: ResMut<WorldBlocks>,
    mut pending_generated: ResMut<PendingGeneratedMaterials>,
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut turn_cache: ResMut<TurnCache>,
    mut turn_deps: SimulationTurnDeps,
) {
    if *builder_mode != BuilderMode::Play {
        return;
    }
    if !simulation.running && !simulation.step_requested {
        return;
    }
    if !turn_cache.needs_prefetch(simulation.turn) {
        return;
    }

    let animation_duration = if simulation.running {
        SIMULATION_TURN_SECONDS / simulation.speed.max(0.001)
    } else {
        SIMULATION_TURN_SECONDS
    };

    while turn_cache.needs_prefetch(simulation.turn) {
        let next_turn = turn_cache.simulated_through.max(simulation.turn) + 1;
        if turn_cache.has_pending_turn(next_turn) {
            break;
        }
        let output = simulate_turn(
            &mut world,
            &mut pending_generated,
            &mut signal_cache,
            next_turn,
            animation_duration,
            &mut turn_deps.structure_state,
            &mut turn_deps.movement_influence,
            &mut turn_deps.pusher_state,
            Some(&mut turn_deps.sim_log),
            None,
        );
        turn_cache.store_prefetch(output);
    }
}

pub fn tick_simulation(
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    world: ResMut<WorldBlocks>,
    mut pending_generated: ResMut<PendingGeneratedMaterials>,
    mut turn_cache: ResMut<TurnCache>,
    mut sim_stats: ResMut<SimulationStepStats>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    pending_previews: Query<Entity, With<PendingGeneratedPreview>>,
    render_assets: Option<Res<WorldRenderAssets>>,
    debug: Res<DebugState>,
    turn_deps: SimulationTurnDeps,
) {
    let Some(render_assets) = render_assets.as_ref() else {
        return;
    };
    if *builder_mode != BuilderMode::Play || (!simulation.running && !simulation.step_requested) {
        prepare_upcoming_generation(&world, &mut pending_generated, simulation.turn + 1);
        refresh_pending_generated_previews(
            &mut commands,
            &mut meshes,
            &pending_previews,
            render_assets,
            &world,
            &pending_generated,
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
        simulation.step_requested = false;
        simulation.accumulator = 0.0;
        if let Some(output) = turn_cache.take_pending(simulation.turn + 1) {
            simulation.turn += 1;
            present_turn(
                output,
                animation_duration_for(simulation.running, simulation.speed),
                &world,
                &mut commands,
                &mut meshes,
                &block_entities,
                render_assets,
                &debug,
                &turn_deps.structure_state,
                &mut sim_stats,
            );
        }
        prepare_upcoming_generation(&world, &mut pending_generated, simulation.turn + 1);
        refresh_pending_generated_previews(
            &mut commands,
            &mut meshes,
            &pending_previews,
            render_assets,
            &world,
            &pending_generated,
            simulation.turn,
            simulation.accumulator,
        );
        return;
    }

    simulation.accumulator += time.delta_secs() * simulation.speed / SIMULATION_TURN_SECONDS;
    while simulation.accumulator >= 1.0 {
        let Some(output) = turn_cache.take_pending(simulation.turn + 1) else {
            break;
        };
        simulation.turn += 1;
        simulation.accumulator -= 1.0;
        present_turn(
            output,
            animation_duration_for(simulation.running, simulation.speed),
            &world,
            &mut commands,
            &mut meshes,
            &block_entities,
            render_assets,
            &debug,
            &turn_deps.structure_state,
            &mut sim_stats,
        );
    }

    prepare_upcoming_generation(&world, &mut pending_generated, simulation.turn + 1);
    refresh_pending_generated_previews(
        &mut commands,
        &mut meshes,
        &pending_previews,
        render_assets,
        &world,
        &pending_generated,
        simulation.turn,
        simulation.accumulator,
    );
}

fn present_turn(
    output: crate::game::simulation::core::TurnOutput,
    animation_duration: f32,
    world: &WorldBlocks,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    sim_stats: &mut SimulationStepStats,
) {
    *sim_stats = output.stats.clone();
    apply_turn_output(
        &output,
        world,
        animation_duration,
        commands,
        meshes,
        block_entities,
        render_assets,
        debug,
        structure_state,
        sim_stats,
    );
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
