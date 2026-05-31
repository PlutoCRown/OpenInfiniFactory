use std::collections::{HashMap, HashSet};
use std::time::Instant;

use bevy::prelude::*;

use crate::game::state::{BuilderMode, SimulationState};
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{
    AnimationTiming, BlockAnimation, BlockAnimationKind, SIMULATION_TURN_SECONDS,
};
use crate::game::world::blocks::BlockData;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_pending_generated_previews, despawn_world,
    rebuild_world_with_runtime_animations_for_debug_state, spawn_pending_generated_block,
    spawn_weld_sparks, BlockEntity, PendingGeneratedPreview, WorldRenderAssets,
};

use super::behaviors::{
    material_source_generation, run_material_behavior_phase, run_weld_behavior_phase,
};
use super::factory_activity::FactoryStructureState;
use super::gravity::mark_gravity_phase;
use super::markers::{run_powered_marker_phase, run_static_marker_phase};
use super::movement::{blocker_animations, mark_structure_movement_phase, PusherState};
pub use super::signals::SignalNetworkCache;
use super::structures::{
    execute_structure_moves_with_pushers, merge_structure_movement_plan, MovementInfluenceCache,
};

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
}

impl PendingGeneratedMaterials {
    pub fn clear(&mut self) {
        self.pending.clear();
        self.pending_destroyed.clear();
    }

    pub(super) fn mark_destroyed(&mut self, pos: IVec3, ready_turn: u64) {
        self.pending_destroyed.entry(pos).or_insert(ready_turn);
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

pub fn run_turn(
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    signal_cache: &mut SignalNetworkCache,
    turn: u64,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    animation_duration: f32,
    debug: &DebugState,
    factory_structures: &mut FactoryStructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
    stats: &mut SimulationStepStats,
) {
    let total_start = Instant::now();
    let mut mark = total_start;
    let mut sample = SimulationStepStats::default();

    world.clear_generated_markers();
    remove_ready_destroyed_materials(world, pending_generated, turn);
    let generated_animations = place_ready_generated_materials(world, pending_generated, turn);
    run_static_marker_phase(world);
    let weld_sparks = run_weld_behavior_phase(world);
    sample.prep_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(world);
    let powered_components = signal_cache.powered_components(world);
    let powered_devices = signal_cache.powered_devices(&powered_components);
    let render_powered_wires = signal_cache.powered_wires(&powered_components);
    sample.signal_ms = mark_elapsed_ms(&mut mark);

    let actuating_devices = pusher_state.actuating_devices(world, &powered_devices);
    let mut movement_plan = mark_gravity_phase(world, factory_structures, &actuating_devices);
    sample.gravity_ms = mark_elapsed_ms(&mut mark);

    run_powered_marker_phase(world, &powered_devices);
    sample.marker_before_move_ms = mark_elapsed_ms(&mut mark);

    let device_movement_plan =
        mark_structure_movement_phase(world, &powered_devices, factory_structures, pusher_state);
    movement_plan = merge_structure_movement_plan(
        movement_plan,
        device_movement_plan,
        world,
        factory_structures,
        movement_influence,
    );
    sample.movement_mark_ms = mark_elapsed_ms(&mut mark);

    let (mut animations, pusher_animations) = execute_structure_moves_with_pushers(
        world,
        movement_plan,
        factory_structures,
        movement_influence,
    );
    merge_generated_animations(&mut animations, generated_animations);
    let mut pusher_animations = pusher_animations
        .into_iter()
        .map(|(pos, mut animation)| {
            animation.duration = animation_duration;
            (pos, animation)
        })
        .collect::<HashMap<_, _>>();
    for (pos, animation) in pusher_state.sustained_animations() {
        pusher_animations.entry(pos).or_insert(animation);
    }
    for (pos, animation) in blocker_animations(world, &powered_devices) {
        pusher_animations.entry(pos).or_insert(animation);
    }
    sample.movement_execute_ms = mark_elapsed_ms(&mut mark);

    run_static_marker_phase(world);
    run_powered_marker_phase(world, &powered_devices);
    sample.marker_after_move_ms = mark_elapsed_ms(&mut mark);

    let drill_sparks = run_material_behavior_phase(
        world,
        &powered_devices,
        factory_structures,
        pending_generated,
        turn + 1,
    );

    prepare_upcoming_generation(world, pending_generated, turn + 1);
    sample.behavior_ms = mark_elapsed_ms(&mut mark);

    signal_cache.refresh(world);
    sample.signal_refresh_ms = mark_elapsed_ms(&mut mark);

    despawn_world(commands, block_entities);
    rebuild_world_with_runtime_animations_for_debug_state(
        commands,
        meshes,
        world,
        render_assets,
        &animations,
        &pusher_animations,
        AnimationTiming::simulation(animation_duration),
        debug,
        factory_structures,
        &render_powered_wires,
    );
    spawn_weld_sparks(commands, render_assets, &weld_sparks);
    spawn_weld_sparks(commands, render_assets, &drill_sparks);
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
    mut meshes: ResMut<Assets<Mesh>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    pending_previews: Query<Entity, With<PendingGeneratedPreview>>,
    render_assets: Res<WorldRenderAssets>,
    debug: Res<DebugState>,
    mut factory_structures: ResMut<FactoryStructureState>,
    mut movement_influence: ResMut<MovementInfluenceCache>,
    mut pusher_state: ResMut<PusherState>,
) {
    if *builder_mode != BuilderMode::Play || (!simulation.running && !simulation.step_requested) {
        prepare_upcoming_generation(&world, &mut pending_generated, simulation.turn + 1);
        refresh_pending_generated_previews(
            &mut commands,
            &mut meshes,
            &pending_previews,
            &render_assets,
            &world,
            &pending_generated,
            simulation.turn,
            simulation.accumulator,
        );
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
            &mut meshes,
            &block_entities,
            &render_assets,
            SIMULATION_TURN_SECONDS,
            &debug,
            &mut factory_structures,
            &mut movement_influence,
            &mut pusher_state,
            &mut sim_stats,
        );
        prepare_upcoming_generation(&world, &mut pending_generated, simulation.turn + 1);
        refresh_pending_generated_previews(
            &mut commands,
            &mut meshes,
            &pending_previews,
            &render_assets,
            &world,
            &pending_generated,
            simulation.turn,
            simulation.accumulator,
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
            &mut meshes,
            &block_entities,
            &render_assets,
            SIMULATION_TURN_SECONDS / simulation.speed.max(0.001),
            &debug,
            &mut factory_structures,
            &mut movement_influence,
            &mut pusher_state,
            &mut sim_stats,
        );
    }

    prepare_upcoming_generation(&world, &mut pending_generated, simulation.turn + 1);
    refresh_pending_generated_previews(
        &mut commands,
        &mut meshes,
        &pending_previews,
        &render_assets,
        &world,
        &pending_generated,
        simulation.turn,
        simulation.accumulator,
    );
}

fn prepare_upcoming_generation(
    world: &WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    ready_turn: u64,
) {
    let blocked_generation: HashSet<IVec3> = pending_generated.pending.keys().copied().collect();
    let generated = material_source_generation(world, ready_turn, &blocked_generation);
    for generated in generated {
        pending_generated
            .pending
            .entry(generated.pos)
            .or_insert(PendingGeneratedMaterial {
                block: generated.block,
                ready_turn,
            });
    }
}

fn place_ready_generated_materials(
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    turn: u64,
) -> HashMap<IVec3, BlockAnimation> {
    let mut ready = Vec::new();
    for (pos, pending) in &pending_generated.pending {
        if pending.ready_turn <= turn {
            ready.push(*pos);
        }
    }

    let mut animations = HashMap::new();
    for pos in ready {
        let Some(pending) = pending_generated.pending.remove(&pos) else {
            continue;
        };
        if world.can_place_platform_at(pos) {
            world.insert(pos, pending.block);
            animations.insert(
                pos,
                BlockAnimation {
                    from_pos: pos,
                    to_pos: pos,
                    from_facing: pending.block.facing,
                    to_facing: pending.block.facing,
                    kind: BlockAnimationKind::SpawnScale,
                    duration: None,
                    progress: None,
                },
            );
        }
    }
    animations
}

fn remove_ready_destroyed_materials(
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    turn: u64,
) {
    let ready: Vec<IVec3> = pending_generated
        .pending_destroyed
        .iter()
        .filter_map(|(pos, ready_turn)| (*ready_turn <= turn).then_some(*pos))
        .collect();
    for pos in ready {
        pending_generated.pending_destroyed.remove(&pos);
        if world.is_material_at(pos) {
            world.remove(&pos);
        }
    }
}

fn merge_generated_animations(
    animations: &mut HashMap<IVec3, BlockAnimation>,
    generated_animations: HashMap<IVec3, BlockAnimation>,
) {
    for (generated_pos, generated_animation) in generated_animations {
        let moved_target = animations.iter().find_map(|(target, animation)| {
            (animation.from_pos == generated_pos).then_some(*target)
        });
        if moved_target.is_none() {
            animations.insert(generated_pos, generated_animation);
        }
    }
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
    spawn_pending_generated_previews(
        commands,
        meshes,
        render_assets,
        world,
        pending_generated,
        turn,
        accumulator,
    );
}

fn spawn_pending_generated_previews(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pending_generated: &PendingGeneratedMaterials,
    turn: u64,
    accumulator: f32,
) {
    for (pos, pending) in &pending_generated.pending {
        let progress = if pending.ready_turn <= turn {
            1.0
        } else if pending.ready_turn == turn + 1 {
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
            *pos,
            pending.block,
            Some(BlockAnimation {
                from_pos: *pos,
                to_pos: *pos,
                from_facing: pending.block.facing,
                to_facing: pending.block.facing,
                kind: BlockAnimationKind::SpawnScale,
                duration: Some(SIMULATION_TURN_SECONDS),
                progress: Some(progress),
            }),
            AnimationTiming::simulation(SIMULATION_TURN_SECONDS),
        );
    }
}

fn mark_elapsed_ms(mark: &mut Instant) -> f64 {
    let now = Instant::now();
    let elapsed = now.saturating_duration_since(*mark).as_secs_f64() * 1000.0;
    *mark = now;
    elapsed
}
