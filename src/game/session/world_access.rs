use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::simulation::movement::PusherState;
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{
    BuilderMode, PendingPlayerSpawn, PlacementState, SimulationState, SolutionState,
};
use crate::game::systems::debug::DebugState;
use crate::game::ui::core::UiNavigation;
use crate::game::ui::{CarriedItem, FreeInventoryTab, InventoryItems};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    BlockEntity, SceneChunkMeshes, WorldRenderAssets, despawn_world, rebuild_world_for_debug_state,
};
use crate::scene::BlockEntityIndex;
use crate::scene::{SceneRenderMut, refresh_edit_changes};
use crate::shared::save::SaveState;

/// 已加载玩法世界及其渲染/模拟 sidecar 的 ECS 访问包
#[derive(SystemParam)]
pub struct PlayingWorldParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub world: ResMut<'w, WorldBlocks>,
    pub render_assets: Option<Res<'w, WorldRenderAssets>>,
    pub debug: Res<'w, DebugState>,
    pub structure_state: ResMut<'w, StructureState>,
    pub movement_influence: ResMut<'w, MovementInfluenceCache>,
    pub pusher_state: ResMut<'w, PusherState>,
    pub block_index: ResMut<'w, BlockEntityIndex>,
    pub scene_chunks: ResMut<'w, SceneChunkMeshes>,
    pub block_entities: Query<'w, 's, Entity, With<BlockEntity>>,
}

impl PlayingWorldParams<'_, '_> {
    /// 编辑后增量刷新受影响格的渲染
    pub fn refresh_edit_changes(&mut self, changed: &HashSet<IVec3>) {
        let Some(render_assets) = self.render_assets.as_ref() else {
            return;
        };
        let mut scene = SceneRenderMut {
            commands: &mut self.commands,
            meshes: &mut self.meshes,
            render_assets,
            block_index: &mut self.block_index,
            scene_chunks: &mut self.scene_chunks,
            debug: &self.debug,
            structure_state: &mut self.structure_state,
        };
        refresh_edit_changes(&mut scene, &self.world, changed);
    }

    /// 清空结构/运动/推杆缓存
    pub fn clear_sim_sidecars(&mut self) {
        self.structure_state.clear();
        self.movement_influence.clear();
        self.pusher_state.clear();
    }

    /// 有渲染资源时拆掉场景并按当前 debug 状态重建
    pub fn rebuild_scene(&mut self) {
        self.structure_state.rebuild_for_runtime(&self.world);
        let Some(render_assets) = self.render_assets.as_ref().map(|assets| (**assets).clone())
        else {
            return;
        };
        despawn_world(
            &mut self.commands,
            &mut self.meshes,
            &self.block_entities,
            &mut self.block_index,
            &mut self.scene_chunks,
        );
        rebuild_world_for_debug_state(
            &mut self.commands,
            &mut self.meshes,
            &self.world,
            &render_assets,
            &self.debug,
            &mut self.structure_state,
            &mut self.block_index,
            &mut self.scene_chunks,
        );
    }
}

/// 会话层存档/模式/背包等状态（不含世界网格与渲染）
#[derive(SystemParam)]
pub struct SessionStateParams<'w> {
    pub builder_mode: ResMut<'w, BuilderMode>,
    pub inventory: ResMut<'w, InventoryItems>,
    pub free_inventory_tab: ResMut<'w, FreeInventoryTab>,
    pub carried: ResMut<'w, CarriedItem>,
    pub placement: ResMut<'w, PlacementState>,
    pub ui_navigation: ResMut<'w, UiNavigation>,
    pub save_state: ResMut<'w, SaveState>,
    pub solution_state: ResMut<'w, SolutionState>,
    pub simulation: ResMut<'w, SimulationState>,
    pub pending_player: ResMut<'w, PendingPlayerSpawn>,
}
