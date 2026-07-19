use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::simulation::core::prepare_upcoming_generation;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::pending::PendingGeneratedMaterials;
use crate::game::simulation::signals::SignalNetworkCache;
use crate::game::simulation::stats::SimulationStepStats;
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{BuilderMode, SimulationState};
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{
    AnimationTiming, BlockAnimation, BlockAnimationKind, SIMULATION_TURN_SECONDS,
};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_pending_generated_previews, spawn_pending_generated_block, PendingGeneratedPreview,
    SceneChunkMeshes, WorldRenderAssets,
};
use crate::scene::{apply_turn_output, BlockEntityIndex};
use super::{CachedTurn, SimSnapshot, SimulationWorker, TurnCache};

/// 表现层提交状态：已提交世界与上次通电电线集
#[derive(Resource, Default)]
pub struct SimulationPresentationState {
    pub committed_world: WorldBlocks,
    pub last_powered_wires: HashSet<IVec3>,
}

/// 将模拟快照写回世界与相关缓存资源
pub fn apply_sim_snapshot(
    snapshot: &SimSnapshot,
    world: &mut WorldBlocks,
    pending_generated: &mut PendingGeneratedMaterials,
    signal_cache: &mut SignalNetworkCache,
    structure_state: &mut StructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
) {
    *world = WorldBlocks(snapshot.world.clone());
    *pending_generated = PendingGeneratedMaterials(snapshot.pending_generated.clone());
    *signal_cache = SignalNetworkCache(snapshot.signal_cache.clone());
    *structure_state = StructureState(snapshot.structure_state.clone());
    *movement_influence = MovementInfluenceCache(snapshot.movement_influence.clone());
    *pusher_state = PusherState(snapshot.pusher_state.clone());
}

/// 轮询后台模拟 worker，吞入结果并同步运行配置
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
        simulation.is_active(),
    );
}

/// 预取下一回合（当前为空实现占位）
pub fn prefetch_simulation_turn(
    _builder_mode: Res<BuilderMode>,
    _simulation: Res<SimulationState>,
    _world: ResMut<WorldBlocks>,
    _pending_generated: ResMut<PendingGeneratedMaterials>,
    _signal_cache: ResMut<SignalNetworkCache>,
    _turn_cache: ResMut<TurnCache>,
) {
}

/// tick_simulation 所需的世界/缓存/渲染依赖集合
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
    scene_chunks: ResMut<'w, SceneChunkMeshes>,
    meshes: ResMut<'w, Assets<Mesh>>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
    debug: Res<'w, DebugState>,
}

/// 按缓存回合推进模拟并刷新生成预览
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
            &HashSet::new(),
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
            present_turn(
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
                &mut deps.scene_chunks,
                render_assets,
                &deps.debug,
                &mut deps.sim_stats,
            );
        }
        prepare_upcoming_generation(
            &deps.world,
            &mut deps.pending_generated,
            simulation.turn + 1,
            &HashSet::new(),
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
    // 每帧最多呈现一回合：多回合连续 present 会在命令未 flush 时改索引，
    // 随后对已排队 despawn 的实体 insert，Bevy 0.19 会直接 panic。
    if simulation.accumulator >= 1.0 {
        if let Some(cached) = deps.turn_cache.take_pending(simulation.turn + 1) {
            simulation.turn += 1;
            simulation.accumulator -= 1.0;
            present_turn(
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
                &mut deps.scene_chunks,
                render_assets,
                &deps.debug,
                &mut deps.sim_stats,
            );
        }
    }

    prepare_upcoming_generation(
        &deps.world,
        &mut deps.pending_generated,
        simulation.turn + 1,
        &HashSet::new(),
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

fn present_turn(
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
    scene_chunks: &mut SceneChunkMeshes,
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
    *sim_stats = SimulationStepStats(cached.output.stats.clone());
    apply_turn_output(
        &before,
        world,
        &cached.output,
        &presentation.last_powered_wires,
        animation_duration,
        commands,
        meshes,
        block_index,
        render_assets,
        debug,
        structure_state,
        sim_stats,
        scene_chunks,
    );
    presentation.last_powered_wires = cached.output.powered_wires.clone();
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
                block_id: block.id,
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
