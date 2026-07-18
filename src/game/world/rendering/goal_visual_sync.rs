//! BuilderMode 切换时刷新验收器游玩/编辑外观

use bevy::prelude::*;

use crate::game::simulation::structure_state::StructureState;
use crate::game::state::BuilderMode;
use crate::game::systems::debug::DebugState;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::render_assets::WorldRenderAssets;
use crate::scene::BlockEntityIndex;

use super::components::BlockEntity;
use super::scene_chunks::SceneChunkMeshes;
use super::world_rebuild::{despawn_world, rebuild_world_for_debug_state};

/// 注册验收器外观与 BuilderMode 的同步
pub fn register_goal_visual_systems(app: &mut App) {
    app.add_systems(Update, sync_goal_play_visual_on_builder_mode);
}

/// BuilderMode 变化时切换验收器外观并重建世界
fn sync_goal_play_visual_on_builder_mode(
    builder_mode: Res<BuilderMode>,
    render_assets: Option<ResMut<WorldRenderAssets>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    world: Res<WorldBlocks>,
    debug: Res<DebugState>,
    structure_state: ResMut<StructureState>,
    mut block_index: ResMut<BlockEntityIndex>,
    mut scene_chunks: ResMut<SceneChunkMeshes>,
) {
    let Some(mut render_assets) = render_assets else {
        return;
    };
    if !render_assets.goal_play_visual_initialized {
        return;
    }
    let play = *builder_mode == BuilderMode::Play;
    if render_assets.goal_play_visual == play {
        return;
    }
    render_assets.goal_play_visual = play;
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
        &world,
        &render_assets,
        &debug,
        &structure_state,
        &mut block_index,
        &mut scene_chunks,
    );
}
