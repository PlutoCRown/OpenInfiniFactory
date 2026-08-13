use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::audio::{PlaySound, SoundId};
use crate::game::simulation::core::PresentationPhase;
use crate::game::simulation::core::{prepare_upcoming_generation, simulate_turn};
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::pending::PendingGeneratedMaterials;
use crate::game::simulation::signals::SignalNetworkCache;
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
    pub last_powered_wires: HashSet<IVec3>,
    pub last_powered_devices: HashSet<IVec3>,
}

/// 已提交的模拟回合：只携带表现所需的前态、输出与播放时长
#[derive(Message)]
pub struct TurnCommitted {
    before: WorldBlocks,
    output: crate::sim_bridge::TurnOutput,
    animation_duration: f32,
}

/// 回合表现所需的场景依赖集合
#[derive(SystemParam)]
pub struct SimulationPresentationDeps<'w> {
    pub(crate) world: ResMut<'w, WorldBlocks>,
    pub(crate) structure_state: ResMut<'w, StructureState>,
    pub(crate) presentation: ResMut<'w, SimulationPresentationState>,
    pub(crate) block_index: ResMut<'w, BlockEntityIndex>,
    pub(crate) scene_chunks: ResMut<'w, SceneChunkMeshes>,
    pub(crate) meshes: ResMut<'w, Assets<Mesh>>,
    pub(crate) render_assets: Option<Res<'w, WorldRenderAssets>>,
    pub(crate) portal_flash_queue: ResMut<'w, PortalFlashQueue>,
    pub(crate) debug: Res<'w, DebugState>,
}

/// 原地推进权威模拟状态并发布已提交回合
pub fn advance_simulation(
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut world: ResMut<WorldBlocks>,
    mut pending_generated: ResMut<PendingGeneratedMaterials>,
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut structure_state: ResMut<StructureState>,
    mut movement_influence: ResMut<MovementInfluenceCache>,
    mut pusher_state: ResMut<PusherState>,
    mut committed_turns: MessageWriter<TurnCommitted>,
) {
    if *builder_mode != BuilderMode::Play || (!simulation.running && !simulation.step_requested) {
        prepare_upcoming_generation(
            &world,
            &mut pending_generated,
            simulation.turn + 1,
            &HashSet::new(),
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
        let next_turn = simulation.turn + 1;
        let before = world.clone();
        let output = simulate_turn(
            &mut world,
            &mut pending_generated,
            &mut signal_cache,
            next_turn,
            &mut structure_state,
            &mut movement_influence,
            &mut pusher_state,
            None,
            None,
        );
        simulation.turn = next_turn;
        committed_turns.write(TurnCommitted {
            before,
            output,
            animation_duration: animation_duration_for(simulation.running, simulation.speed),
        });
        prepare_upcoming_generation(
            &world,
            &mut pending_generated,
            simulation.turn + 1,
            &HashSet::new(),
        );
        return;
    }

    simulation.accumulator += time.delta_secs() * simulation.speed / SIMULATION_TURN_SECONDS;
    // 每帧最多呈现一回合：多回合连续 present 会在命令未 flush 时改索引，
    // 随后对已排队 despawn 的实体 insert，Bevy 0.19 会直接 panic。
    if simulation.accumulator >= 1.0 {
        let next_turn = simulation.turn + 1;
        let before = world.clone();
        let output = simulate_turn(
            &mut world,
            &mut pending_generated,
            &mut signal_cache,
            next_turn,
            &mut structure_state,
            &mut movement_influence,
            &mut pusher_state,
            None,
            None,
        );
        simulation.turn = next_turn;
        simulation.accumulator -= 1.0;
        committed_turns.write(TurnCommitted {
            before,
            output,
            animation_duration: animation_duration_for(simulation.running, simulation.speed),
        });
    }

    prepare_upcoming_generation(
        &world,
        &mut pending_generated,
        simulation.turn + 1,
        &HashSet::new(),
    );
}

/// 消费已提交回合并更新场景、动画、音效和表现统计
pub fn present_simulation_turns(
    mut committed_turns: MessageReader<TurnCommitted>,
    mut commands: Commands,
    mut sounds: MessageWriter<PlaySound>,
    mut deps: SimulationPresentationDeps,
) {
    let Some(render_assets) = deps.render_assets.as_ref() else {
        for _ in committed_turns.read() {}
        return;
    };
    for committed in committed_turns.read() {
        let output = &committed.output;
        let mut presentation_stats = output.stats.clone();
        let mut scene = SceneRenderMut {
            commands: &mut commands,
            meshes: &mut deps.meshes,
            render_assets,
            block_index: &mut deps.block_index,
            scene_chunks: &mut deps.scene_chunks,
            debug: &deps.debug,
            structure_state: &mut deps.structure_state,
        };
        apply_turn_output(
            &committed.before,
            &deps.world,
            output,
            &deps.presentation.last_powered_wires,
            committed.animation_duration,
            &mut scene,
            &mut presentation_stats,
            &mut deps.portal_flash_queue,
        );
        commands.insert_resource(presentation_stats);
        let turn_start =
            output.presentation_delay(PresentationPhase::TurnStart, committed.animation_duration);
        let after_motion =
            output.presentation_delay(PresentationPhase::AfterMotion, committed.animation_duration);
        for &(from, to) in &output.weld_sparks {
            sounds.write(PlaySound {
                sound: SoundId::Weld,
                position: Some(grid_to_world(from).lerp(grid_to_world(to), 0.5)),
                gain: 1.0,
                delay: after_motion,
                speed: 1.0,
            });
        }
        for debris in &output.break_debris {
            sounds.write(PlaySound {
                sound: SoundId::DrillBreak,
                position: Some(grid_to_world(debris.pos)),
                gain: 1.0,
                delay: turn_start,
                speed: 1.0,
            });
        }
        for debris in &output.acceptance_sparks {
            sounds.write(PlaySound {
                sound: SoundId::Acceptance,
                position: Some(grid_to_world(debris.pos)),
                gain: 1.0,
                delay: turn_start,
                speed: 1.0,
            });
        }
        for &(from, to, _) in &output.teleport_flashes {
            sounds.write(PlaySound {
                sound: SoundId::SciFi,
                position: Some(grid_to_world(from).lerp(grid_to_world(to), 0.5)),
                gain: 1.0,
                delay: turn_start,
                speed: 1.0,
            });
        }
        for &pos in &output.behavior_sparks {
            sounds.write(PlaySound {
                sound: SoundId::MachineWork,
                position: Some(grid_to_world(pos)),
                gain: 0.7,
                delay: turn_start,
                speed: 1.0,
            });
        }
        for (&pos, motion) in &output.pusher_animations {
            if motion.from_extension == motion.to_extension {
                continue;
            }
            sounds.write(PlaySound {
                sound: if motion.to_extension > motion.from_extension {
                    SoundId::PusherExtend
                } else {
                    SoundId::PusherRetract
                },
                position: Some(grid_to_world(pos)),
                gain: 1.0,
                delay: turn_start,
                speed: SIMULATION_TURN_SECONDS / committed.animation_duration.max(f32::EPSILON),
            });
        }
        for &pos in &output.powered_devices {
            let Some(block) = deps.world.blocks.get(&pos) else {
                continue;
            };
            if matches!(
                block.kind,
                oif_sim::blocks::BlockKind::Rotator
                    | oif_sim::blocks::BlockKind::CounterRotator
                    | oif_sim::blocks::BlockKind::Lifter
            ) {
                sounds.write(PlaySound {
                    sound: SoundId::MachineWork,
                    position: Some(grid_to_world(pos)),
                    gain: 0.75,
                    delay: turn_start,
                    speed: 1.0,
                });
            }
        }
        deps.presentation.last_powered_wires = output.powered_wires.clone();
        deps.presentation.last_powered_devices = output.powered_devices.clone();
    }
}

/// 按当前权威状态刷新生成材料预览
pub fn refresh_pending_generated_previews(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    pending_previews: Query<Entity, With<PendingGeneratedPreview>>,
    render_assets: Option<Res<WorldRenderAssets>>,
    world: Res<WorldBlocks>,
    pending_generated: Res<PendingGeneratedMaterials>,
    simulation: Res<SimulationState>,
) {
    let Some(render_assets) = render_assets else {
        return;
    };
    despawn_pending_generated_previews(&mut commands, &pending_previews);
    for (pos, block, ready_turn) in pending_generated.pending_entries() {
        let progress = if ready_turn <= simulation.turn {
            1.0
        } else if ready_turn == simulation.turn + 1 {
            simulation.accumulator
        } else {
            0.0
        }
        .clamp(0.0, 1.0);

        spawn_pending_generated_block(
            &mut commands,
            &mut meshes,
            &render_assets,
            &world,
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
