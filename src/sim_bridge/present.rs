use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

use super::{CachedTurn, SimSnapshot, SimulationWorker, TurnCache};
use crate::game::audio::{PlaySound, SoundId};
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
use crate::game::world::grid::{WorldBlocks, grid_to_world};
use crate::game::world::rendering::{
    PendingGeneratedPreview, PortalFlashQueue, SceneChunkMeshes, WorldRenderAssets,
    despawn_pending_generated_previews, spawn_pending_generated_block,
};
use crate::scene::{BlockEntityIndex, SceneRenderMut, apply_turn_output};

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
    pub(crate) world: ResMut<'w, WorldBlocks>,
    pub(crate) pending_generated: ResMut<'w, PendingGeneratedMaterials>,
    pub(crate) signal_cache: ResMut<'w, SignalNetworkCache>,
    pub(crate) structure_state: ResMut<'w, StructureState>,
    pub(crate) movement_influence: ResMut<'w, MovementInfluenceCache>,
    pub(crate) pusher_state: ResMut<'w, PusherState>,
    pub(crate) turn_cache: ResMut<'w, TurnCache>,
    pub(crate) sim_stats: ResMut<'w, SimulationStepStats>,
    pub(crate) presentation: ResMut<'w, SimulationPresentationState>,
    pub(crate) block_index: ResMut<'w, BlockEntityIndex>,
    pub(crate) scene_chunks: ResMut<'w, SceneChunkMeshes>,
    pub(crate) meshes: ResMut<'w, Assets<Mesh>>,
    pub(crate) render_assets: Option<Res<'w, WorldRenderAssets>>,
    pub(crate) portal_flash_queue: ResMut<'w, PortalFlashQueue>,
    pub(crate) debug: Res<'w, DebugState>,
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
    if *builder_mode != BuilderMode::Play || (!simulation.running && !simulation.step_requested) {
        if deps.render_assets.is_none() {
            return;
        }
        let render_assets = deps.render_assets.as_ref().unwrap();
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

    if deps.render_assets.is_none() {
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
                &mut simulation.last_powered_devices,
                &mut deps,
                &mut commands,
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
            deps.render_assets.as_ref().unwrap(),
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
                &mut simulation.last_powered_devices,
                &mut deps,
                &mut commands,
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
        deps.render_assets.as_ref().unwrap(),
        &deps.world,
        &deps.pending_generated,
        simulation.turn,
        simulation.accumulator,
    );
}

fn present_turn(
    cached: CachedTurn,
    animation_duration: f32,
    last_powered_devices: &mut HashSet<IVec3>,
    deps: &mut SimulationTickDeps,
    commands: &mut Commands,
) {
    let Some(render_assets) = deps.render_assets.as_ref() else {
        return;
    };
    let before = deps.presentation.committed_world.clone();
    apply_sim_snapshot(
        &cached.after,
        &mut deps.world,
        &mut deps.pending_generated,
        &mut deps.signal_cache,
        &mut deps.structure_state,
        &mut deps.movement_influence,
        &mut deps.pusher_state,
    );
    *deps.sim_stats = SimulationStepStats(cached.output.stats.clone());
    let mut scene = SceneRenderMut {
        commands,
        meshes: &mut deps.meshes,
        render_assets,
        block_index: &mut deps.block_index,
        scene_chunks: &mut deps.scene_chunks,
        debug: &deps.debug,
        structure_state: &mut deps.structure_state,
    };
    apply_turn_output(
        &before,
        &deps.world,
        &cached.output,
        &deps.presentation.last_powered_wires,
        animation_duration,
        &mut scene,
        &mut deps.sim_stats,
        &mut deps.portal_flash_queue,
    );
    for &(from, to) in &cached.output.weld_sparks {
        commands.write_message(PlaySound {
            sound: SoundId::Weld,
            position: Some(grid_to_world(from).lerp(grid_to_world(to), 0.5)),
            gain: 1.0,
        });
    }
    for debris in &cached.output.break_debris {
        commands.write_message(PlaySound {
            sound: SoundId::DrillBreak,
            position: Some(grid_to_world(debris.pos)),
            gain: 1.0,
        });
    }
    for debris in &cached.output.acceptance_sparks {
        commands.write_message(PlaySound {
            sound: SoundId::Acceptance,
            position: Some(grid_to_world(debris.pos)),
            gain: 1.0,
        });
    }
    for &(from, to, _) in &cached.output.teleport_flashes {
        commands.write_message(PlaySound {
            sound: SoundId::SciFi,
            position: Some(grid_to_world(from).lerp(grid_to_world(to), 0.5)),
            gain: 1.0,
        });
    }
    for &pos in &cached.output.behavior_sparks {
        commands.write_message(PlaySound {
            sound: SoundId::MachineWork,
            position: Some(grid_to_world(pos)),
            gain: 0.7,
        });
    }
    for &pos in cached.output.pusher_animations.keys() {
        commands.write_message(PlaySound {
            sound: SoundId::MachineWork,
            position: Some(grid_to_world(pos)),
            gain: 1.0,
        });
    }
    for &pos in &cached.output.powered_devices {
        let Some(block) = deps.world.blocks.get(&pos) else {
            continue;
        };
        if matches!(
            block.kind,
            oif_sim::blocks::BlockKind::Rotator
                | oif_sim::blocks::BlockKind::CounterRotator
                | oif_sim::blocks::BlockKind::Lifter
                | oif_sim::blocks::BlockKind::Welder
                | oif_sim::blocks::BlockKind::DownWelder
        ) {
            commands.write_message(PlaySound {
                sound: SoundId::MachineWork,
                position: Some(grid_to_world(pos)),
                gain: 0.75,
            });
        }
    }
    deps.presentation.last_powered_wires = cached.output.powered_wires.clone();
    *last_powered_devices = cached.output.powered_devices.clone();
    deps.presentation.committed_world = deps.world.clone();
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
