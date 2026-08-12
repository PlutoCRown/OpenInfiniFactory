use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::pending::PendingGeneratedMaterials;
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{BuilderMode, GameMode, SimulationState};
use crate::game::systems::debug::DebugState;
use crate::game::ui::UiNavigation;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    BlockEntity, GeneratorConfigMaterialPreview, SceneChunkMeshes, WorldRenderAssets,
    despawn_world, rebuild_world_for_debug_state,
};
use crate::sim_bridge::SimulationPresentationState;
use crate::sim_bridge::reset_simulation_presentation;

#[derive(SystemParam)]
pub struct SimulationControlDeps<'w> {
    builder_mode: Res<'w, BuilderMode>,
    mode: Res<'w, State<GameMode>>,
    ui_navigation: Res<'w, UiNavigation>,
    simulation: ResMut<'w, SimulationState>,
    pending_generated: ResMut<'w, PendingGeneratedMaterials>,
    structure_state: ResMut<'w, StructureState>,
    movement_influence: ResMut<'w, MovementInfluenceCache>,
    pusher_state: ResMut<'w, PusherState>,
    world: ResMut<'w, WorldBlocks>,
    presentation: ResMut<'w, SimulationPresentationState>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
    debug: Res<'w, DebugState>,
}

pub fn simulation_controls(
    input: Res<crate::game::input::GameplayInputState>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut deps: SimulationControlDeps,
    mut block_index: ResMut<crate::scene::BlockEntityIndex>,
    mut scene_chunks: ResMut<SceneChunkMeshes>,
    // 用 F 从停/暂停启动后，同一次按住不算加速，松手后再按才加速
    mut suppress_sim_fast_until_release: Local<bool>,
) {
    if *deps.builder_mode != BuilderMode::Play
        || *deps.mode.get() != GameMode::Playing
        || !deps.ui_navigation.active_play()
        || deps.ui_navigation.blocks_gameplay()
    {
        return;
    }
    let Some(render_assets) = deps.render_assets.as_ref() else {
        return;
    };

    if input.simulate {
        let was_running = deps.simulation.running;
        deps.simulation.run(
            &deps.world,
            &mut deps.structure_state,
            &mut deps.pusher_state,
        );
        reset_simulation_presentation(&mut deps.presentation);
        // 未运行/暂停时按 F 是启动；与加速同键时需松手再按才加速
        if !was_running {
            *suppress_sim_fast_until_release = true;
        }
    }

    if input.sim_step {
        if !deps.simulation.is_active() {
            return;
        }
        if deps.simulation.running {
            deps.simulation.pause();
        } else {
            deps.simulation.step_requested = true;
        }
    }

    if !input.sim_fast {
        *suppress_sim_fast_until_release = false;
    }

    // 单步模拟暂停时，触控加速键等同于 F，恢复连续模拟。
    if input.sim_fast && deps.simulation.is_active() && !deps.simulation.running {
        deps.simulation.run(
            &deps.world,
            &mut deps.structure_state,
            &mut deps.pusher_state,
        );
        *suppress_sim_fast_until_release = true;
    }

    deps.simulation.speed =
        if deps.simulation.running && input.sim_fast && !*suppress_sim_fast_until_release {
            4.0
        } else {
            1.0
        };

    if input.rollback && deps.simulation.is_active() {
        deps.simulation.rollback(
            &mut deps.world,
            &mut deps.pending_generated,
            &mut deps.structure_state,
            &mut deps.movement_influence,
            &mut deps.pusher_state,
        );
        refresh_static_generated_markers(&mut deps.world);
        reset_simulation_presentation(&mut deps.presentation);
        despawn_world(
            &mut commands,
            &mut meshes,
            &block_entities,
            &mut block_index,
            &mut scene_chunks,
        );
        rebuild_world_for_debug_state(
            &mut commands,
            &mut meshes,
            &deps.world,
            render_assets,
            &deps.debug,
            &deps.structure_state,
            &mut block_index,
            &mut scene_chunks,
        );
    }
}

/// 模拟激活时隐藏生成块配置材料小预览，退出后恢复
pub fn sync_generator_config_material_preview(
    simulation: Res<SimulationState>,
    mut was_active: Local<Option<bool>>,
    mut previews: Query<&mut Visibility, With<GeneratorConfigMaterialPreview>>,
) {
    let active = simulation.is_active();
    if *was_active == Some(active) {
        return;
    }
    *was_active = Some(active);
    let visibility = if active {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut preview_visibility in &mut previews {
        *preview_visibility = visibility;
    }
}
