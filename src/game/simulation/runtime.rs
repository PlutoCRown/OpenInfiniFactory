use std::time::Instant;
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::state::{BuilderMode, SimulationState};
use crate::game::world::animation::{
    AnimationTiming, BlockAnimation, BlockAnimationKind, SIMULATION_TURN_SECONDS,
};
use crate::game::world::blocks::BlockData;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_pending_generated_previews, despawn_world, rebuild_world_with_runtime_animations,
    spawn_pending_generated_block, BlockEntity, PendingGeneratedPreview, WorldRenderAssets,
};

use super::behaviors::run_material_behavior_phase;
use super::gravity::mark_gravity_phase;
use super::markers::{run_powered_marker_phase, run_static_marker_phase};
use super::movement::mark_material_movement_phase;
pub use super::signals::SignalNetworkCache;
use super::structures::execute_structure_moves_with_pistons;

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
}

impl PendingGeneratedMaterials {
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

struct PendingGeneratedMaterial {
    block: BlockData,
    duration: f32,
    remaining: f32,
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

pub fn run_turn(
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    animation_duration: f32,
    stats: &mut SimulationStepStats,
) {
    let total_start = Instant::now();
    let mut mark = total_start;
    let mut sample = SimulationStepStats::default();

    world.clear_generated_markers();
    sample.prep_ms = mark_elapsed_ms(&mut mark);

    let mut movement_plan = mark_gravity_phase(world);
    sample.gravity_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(world);
    let powered_components = signal_cache.powered_components(world);
    let powered_devices = signal_cache.powered_devices(&powered_components);
    sample.signal_ms = mark_elapsed_ms(&mut mark);

    run_powered_marker_phase(world, &powered_devices);
    sample.marker_before_move_ms = mark_elapsed_ms(&mut mark);

    movement_plan.extend(mark_material_movement_phase(world, &powered_devices));
    sample.movement_mark_ms = mark_elapsed_ms(&mut mark);

    let (animations, piston_animations) = execute_structure_moves_with_pistons(world, movement_plan);
    let piston_animations = piston_animations
        .into_iter()
        .map(|(pos, mut animation)| {
            animation.duration = animation_duration;
            (pos, animation)
        })
        .collect();
    sample.movement_execute_ms = mark_elapsed_ms(&mut mark);

    run_static_marker_phase(world);
    run_powered_marker_phase(world, &powered_devices);
    sample.marker_after_move_ms = mark_elapsed_ms(&mut mark);

    let blocked_generation: HashSet<IVec3> = pending_generated.pending.keys().copied().collect();
    let generated = run_material_behavior_phase(world, turn, &powered_devices, &blocked_generation);
    for generated in generated {
        pending_generated.pending.insert(
            generated.pos,
            PendingGeneratedMaterial {
                block: generated.block,
                duration: animation_duration * generated.period as f32,
                remaining: animation_duration * generated.period as f32,
            },
        );
    }
    sample.behavior_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(world);
    sample.signal_refresh_ms = mark_elapsed_ms(&mut mark);

    despawn_world(commands, block_entities);
    rebuild_world_with_runtime_animations(
        commands,
        world,
        render_assets,
        &animations,
        &piston_animations,
        AnimationTiming::simulation(animation_duration),
    );
    sample.render_rebuild_ms = mark_elapsed_ms(&mut mark);
    sample.total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    sample.has_sample = true;
    *stats = sample;
}

pub fn tick_simulation(
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut world: ResMut<WorldBlocks>,
    mut pending_generated: ResMut<PendingGeneratedMaterials>,
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut sim_stats: ResMut<SimulationStepStats>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    pending_previews: Query<Entity, With<PendingGeneratedPreview>>,
    render_assets: Res<WorldRenderAssets>,
) {
    despawn_pending_generated_previews(&mut commands, &pending_previews);
    tick_pending_generated(time.delta_secs(), &mut world, &mut pending_generated);
    spawn_pending_generated_previews(&mut commands, &render_assets, &world, &pending_generated);

    if *builder_mode != BuilderMode::Play || (!simulation.running && !simulation.step_requested) {
        return;
    }

    if simulation.step_requested {
        simulation.step_requested = false;
        simulation.accumulator = 0.0;
        simulation.turn += 1;
        run_turn(
            &mut world,
            &mut pending_generated,
            &mut signal_cache,
            simulation.turn,
            &mut commands,
            &block_entities,
            &render_assets,
            SIMULATION_TURN_SECONDS,
            &mut sim_stats,
        );
        return;
    }

    simulation.accumulator += time.delta_secs() * simulation.speed / SIMULATION_TURN_SECONDS;
    while simulation.accumulator >= 1.0 {
        simulation.turn += 1;
        simulation.accumulator -= 1.0;
        run_turn(
            &mut world,
            &mut pending_generated,
            &mut signal_cache,
            simulation.turn,
            &mut commands,
            &block_entities,
            &render_assets,
            SIMULATION_TURN_SECONDS / simulation.speed.max(0.001),
            &mut sim_stats,
        );
    }
}

fn tick_pending_generated(
    delta_seconds: f32,
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
) {
    let mut ready = Vec::new();
    for (pos, pending) in &mut pending_generated.pending {
        pending.remaining -= delta_seconds;
        if pending.remaining <= 0.0 {
            ready.push(*pos);
        }
    }

    for pos in ready {
        let Some(pending) = pending_generated.pending.remove(&pos) else {
            continue;
        };
        if world.can_place_solid_at(pos) {
            world.insert(pos, pending.block);
        }
    }
}

fn spawn_pending_generated_previews(
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pending_generated: &PendingGeneratedMaterials,
) {
    for (pos, pending) in &pending_generated.pending {
        spawn_pending_generated_block(
            commands,
            render_assets,
            world,
            *pos,
            pending.block,
            Some(BlockAnimation {
                from_pos: *pos,
                to_pos: *pos,
                from_facing: pending.block.facing,
                to_facing: pending.block.facing,
                kind: BlockAnimationKind::SpawnScale,
                duration: Some(pending.duration),
                progress: Some(1.0 - pending.remaining / pending.duration.max(f32::EPSILON)),
            }),
            AnimationTiming::simulation(pending.remaining.max(0.0)),
        );
    }
}

fn mark_elapsed_ms(mark: &mut Instant) -> f64 {
    let now = Instant::now();
    let elapsed = now.saturating_duration_since(*mark).as_secs_f64() * 1000.0;
    *mark = now;
    elapsed
}
