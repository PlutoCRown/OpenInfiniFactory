use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::simulation::movement::PusherState;
use crate::game::simulation::pending::PendingTurnEffects;
use crate::game::simulation::signals::SignalNetworkCache;
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementHistory;
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

/// 已加载玩法世界、模拟状态与渲染索引的 ECS 访问包
#[derive(SystemParam)]
pub struct PlayingWorldParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub world: ResMut<'w, WorldBlocks>,
    pub render_assets: Option<Res<'w, WorldRenderAssets>>,
    pub debug: Res<'w, DebugState>,
    pub structure_state: ResMut<'w, StructureState>,
    pub movement_history: ResMut<'w, MovementHistory>,
    pub pusher_state: ResMut<'w, PusherState>,
    pub pending_effects: ResMut<'w, PendingTurnEffects>,
    pub signal_cache: ResMut<'w, SignalNetworkCache>,
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

    /// 世界替换时统一重置模拟历史、待执行操作和派生索引
    pub fn reset_simulation_state(&mut self) {
        self.structure_state.clear();
        self.movement_history.clear();
        self.pusher_state.clear();
        self.pending_effects.clear();
        *self.signal_cache = SignalNetworkCache::default();
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

/// 游戏会话切换时的状态清理回归
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;
    use oif_sim::blocks::{BlockData, BlockKind};
    use oif_sim::world::Facing;
    use oif_sim::world::grid::{GeneratorMode, GeneratorSettings};

    /// 通过实际 ECS 访问包清空挂起操作和结构，验证资源组合没有借用冲突
    #[test]
    fn reset_clears_pending_effects_and_structure_state() {
        let mut ecs = World::new();
        ecs.init_resource::<Assets<Mesh>>();
        ecs.init_resource::<WorldBlocks>();
        ecs.init_resource::<DebugState>();
        ecs.init_resource::<StructureState>();
        ecs.init_resource::<MovementHistory>();
        ecs.init_resource::<PusherState>();
        ecs.init_resource::<PendingTurnEffects>();
        ecs.init_resource::<SignalNetworkCache>();
        ecs.init_resource::<BlockEntityIndex>();
        ecs.init_resource::<SceneChunkMeshes>();
        let mut access = SystemState::<PlayingWorldParams>::new(&mut ecs);
        {
            let mut playing = access.get_mut(&mut ecs).unwrap();
            playing.world.insert(
                IVec3::ZERO,
                BlockData::new(BlockKind::Generator, Facing::North),
            );
            playing.world.set_generator_settings(
                IVec3::ZERO,
                GeneratorSettings {
                    mode: GeneratorMode::Period {
                        period: 1,
                        offset: 0,
                    },
                    ..Default::default()
                },
            );
            playing
                .world
                .insert(IVec3::X, BlockData::new(BlockKind::Wire, Facing::North));
            playing
                .pending_effects
                .schedule_generation(&playing.world, 1, &HashSet::new());
            playing.structure_state.rebuild_for_runtime(&playing.world);
            playing.signal_cache.refresh(&playing.world);
            assert_eq!(playing.pending_effects.pending_entries().count(), 1);
            assert!(!playing.structure_state.is_empty());
            playing.reset_simulation_state();
            assert_eq!(playing.pending_effects.pending_entries().count(), 0);
            assert!(playing.structure_state.is_empty());
        }
        access.apply(&mut ecs);
    }
}
