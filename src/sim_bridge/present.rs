use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::audio::{PlaySound, SoundId};
use crate::game::simulation::core::PresentationPhase;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::pending::PendingTurnEffects;
use crate::game::simulation::signals::SignalNetworkCache;
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementHistory;
use crate::game::state::{BuilderMode, SimulationState};
use crate::game::world::animation::{
    AnimationTiming, BlockAnimation, BlockAnimationKind, SIMULATION_TURN_SECONDS,
};
use crate::game::world::grid::{WorldBlocks, grid_to_world};
use crate::game::world::rendering::{
    PendingGeneratedPreview, PortalFlashQueue, SceneChunkMeshes, WorldRenderAssets,
    spawn_pending_generated_block,
};
use crate::scene::{BlockEntityIndex, SceneRenderMut, apply_turn_output};
use oif_sim::simulation::core::TurnRunner;

/// 表现层提交状态：已提交世界与上次通电电线集
#[derive(Resource, Default)]
pub struct SimulationPresentationState {
    pub last_powered_wires: HashSet<IVec3>,
    pub last_powered_devices: HashSet<IVec3>,
}

/// 已提交的模拟回合：只携带表现所需的前态、输出与播放时长
#[derive(Message)]
pub struct TurnCommitted {
    epoch: oif_sim::session::SessionEpoch,
    before: WorldBlocks,
    output: crate::sim_bridge::TurnOutput,
    animation_duration: f32,
}

/// 回合表现所需的场景依赖集合
#[derive(SystemParam)]
pub struct SimulationPresentationDeps<'w> {
    pub(crate) world: Res<'w, WorldBlocks>,
    pub(crate) presentation: ResMut<'w, SimulationPresentationState>,
    pub(crate) block_index: ResMut<'w, BlockEntityIndex>,
    pub(crate) scene_chunks: ResMut<'w, SceneChunkMeshes>,
    pub(crate) meshes: ResMut<'w, Assets<Mesh>>,
    pub(crate) render_assets: Option<Res<'w, WorldRenderAssets>>,
    pub(crate) portal_flash_queue: ResMut<'w, PortalFlashQueue>,
}

/// 原地推进权威模拟状态并发布已提交回合
pub fn advance_simulation(
    mut runner: ResMut<TurnRunner>,
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut world: ResMut<WorldBlocks>,
    mut pending_effects: ResMut<PendingTurnEffects>,
    mut signal_cache: ResMut<SignalNetworkCache>,
    mut structure_state: ResMut<StructureState>,
    mut movement_history: ResMut<MovementHistory>,
    mut pusher_state: ResMut<PusherState>,
    mut committed_turns: MessageWriter<TurnCommitted>,
) {
    if *builder_mode != BuilderMode::Play || (!simulation.running && !simulation.step_requested) {
        return;
    }

    if simulation.step_requested {
        simulation.step_requested = false;
        simulation.accumulator = 0.0;
    } else {
        simulation.accumulator += time.delta_secs() * simulation.speed / SIMULATION_TURN_SECONDS;
        // 每帧最多推进一回合，保证表现命令落地后才能更新下一次实体索引。
        if simulation.accumulator < 1.0 {
            return;
        }
        simulation.accumulator -= 1.0;
    }
    let next_turn = simulation.turn + 1;
    let before = world.clone();
    let output = runner.run(
        &mut world,
        &mut pending_effects,
        &mut signal_cache,
        next_turn,
        &mut structure_state,
        &mut movement_history,
        &mut pusher_state,
        None,
        None,
    );
    simulation.turn = next_turn;
    committed_turns.write(TurnCommitted {
        epoch: simulation.epoch().clone(),
        before,
        output,
        animation_duration: if simulation.running {
            SIMULATION_TURN_SECONDS / simulation.speed.max(0.001)
        } else {
            SIMULATION_TURN_SECONDS
        },
    });
}

/// 消费已提交回合并更新场景、动画、音效和表现统计
pub fn present_simulation_turns(
    simulation: Res<SimulationState>,
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
        if !simulation.epoch().matches(&committed.epoch) || simulation.turn != committed.output.turn
        {
            continue;
        }
        let output = &committed.output;
        let mut presentation_stats = output.stats.clone();
        let mut scene = SceneRenderMut {
            commands: &mut commands,
            meshes: &mut deps.meshes,
            render_assets,
            block_index: &mut deps.block_index,
            scene_chunks: &mut deps.scene_chunks,
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
            let Some(block) = deps.world.blocks().get(&pos) else {
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

/// 将编辑预测或已提交操作投影到预览实体；只有模拟队列会影响材料落地
pub fn refresh_pending_generated_previews(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pending_previews: Query<(Entity, &PendingGeneratedPreview, &mut Transform)>,
    render_assets: Option<Res<WorldRenderAssets>>,
    world: Res<WorldBlocks>,
    pending_effects: Res<PendingTurnEffects>,
    simulation: Res<SimulationState>,
    mut committed_source: Local<Option<bool>>,
) {
    let Some(render_assets) = render_assets else {
        return;
    };
    let committed = simulation.turn > 0;
    let reconcile = *committed_source != Some(committed)
        || render_assets.is_changed()
        || if committed {
            pending_effects.is_changed()
        } else {
            world.is_changed()
        };
    *committed_source = Some(committed);
    // 每帧只推进已有实体的生长；增删对齐时才构建待生成位置表。
    let mut remaining: HashMap<IVec3, _> = if reconcile {
        if committed {
            pending_effects
                .pending_entries()
                .map(|(pos, block, ready_turn)| (pos, (block, ready_turn)))
                .collect()
        } else {
            oif_sim::simulation::planned_generation(&world, 1, &HashSet::new())
                .into_iter()
                .map(|generated| (generated.pos, (generated.block, 1)))
                .collect()
        }
    } else {
        HashMap::new()
    };
    for (entity, preview, mut transform) in &mut pending_previews {
        if reconcile {
            if remaining.get(&preview.pos) != Some(&(preview.block, preview.ready_turn)) {
                commands.entity(entity).despawn();
                continue;
            }
            remaining.remove(&preview.pos);
        }
        let progress = if preview.ready_turn <= simulation.turn {
            1.0
        } else if preview.ready_turn == simulation.turn + 1 {
            simulation.accumulator
        } else {
            0.0
        }
        .clamp(0.0, 1.0);
        let scale = Vec3::splat(progress);
        if transform.scale != scale {
            transform.scale = scale;
        }
    }
    for (pos, (block, ready_turn)) in remaining {
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
            ready_turn,
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

#[cfg(test)]
mod tests {
    use super::*;
    use oif_sim::blocks::{BlockData, BlockKind};
    use oif_sim::session::SimSession;
    use oif_sim::world::Facing;

    /// 实际 GUI 推进系统和无头会话共用回合语义，并给消息绑定正确的会话身份。
    #[test]
    fn gui_step_matches_headless_turn_and_tags_commit() {
        let mut session = SimSession::new();
        session
            .world
            .insert(IVec3::Y, BlockData::new(BlockKind::Blocker, Facing::East));
        session.world.insert(
            IVec3::Y + IVec3::X,
            BlockData::new(BlockKind::Platform, Facing::North),
        );
        session.begin_simulation();
        let mut control = session.control.clone();
        control.step_requested = true;
        let mut app = App::new();
        app.init_resource::<TurnRunner>()
            .init_resource::<Time>()
            .insert_resource(BuilderMode::Play)
            .insert_resource(control)
            .insert_resource(session.world.clone())
            .insert_resource(session.structure_state.clone())
            .insert_resource(session.pusher_state.clone())
            .init_resource::<PendingTurnEffects>()
            .init_resource::<SignalNetworkCache>()
            .init_resource::<MovementHistory>()
            .add_message::<TurnCommitted>()
            .add_systems(Update, advance_simulation);
        app.update();
        let expected = session.simulate_next_turn_with_logging(false);
        let world = app.world();
        assert_eq!(
            world.resource::<WorldBlocks>().blocks(),
            session.world.blocks()
        );
        let control = world.resource::<SimulationState>();
        assert_eq!(control.turn, 1);
        assert!(!control.step_requested);
        let messages = world.resource::<Messages<TurnCommitted>>();
        let mut cursor = messages.get_cursor();
        let committed: Vec<_> = cursor.read(messages).collect();
        assert_eq!(committed.len(), 1);
        assert!(control.epoch().matches(&committed[0].epoch));
        assert_eq!(
            committed[0].output.powered_devices,
            expected.powered_devices
        );
        let old_epoch = committed[0].epoch.clone();
        app.world_mut().resource_mut::<SimulationState>().reset();
        assert!(
            !app.world()
                .resource::<SimulationState>()
                .epoch()
                .matches(&old_epoch)
        );
    }
}
