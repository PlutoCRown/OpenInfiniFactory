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
use crate::game::ui::core::UiNavigation;
use crate::game::ui::{CarriedItem, FreeInventoryTab, InventoryItems};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    BlockEntity, SceneChunkMeshes, WorldRenderAssets, despawn_world, rebuild_world,
};
use crate::scene::BlockEntityIndex;
use crate::scene::{SceneRenderMut, refresh_edit_changes};
use crate::shared::save::SaveState;

/// 世界编辑及其增量渲染所需的最小 ECS 访问包
#[derive(SystemParam)]
pub struct EditableWorldParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub world: ResMut<'w, WorldBlocks>,
    pub render_assets: Option<Res<'w, WorldRenderAssets>>,
    pub structure_state: ResMut<'w, StructureState>,
    pub movement_history: ResMut<'w, MovementHistory>,
    pub pusher_state: ResMut<'w, PusherState>,
    pub block_index: ResMut<'w, BlockEntityIndex>,
    pub scene_chunks: ResMut<'w, SceneChunkMeshes>,
}

impl EditableWorldParams<'_, '_> {
    /// 编辑后先维护逻辑结构，再在渲染资源可用时刷新场景
    pub fn refresh_edit_changes(&mut self, changed: &HashSet<IVec3>) {
        refresh_edit_world(
            &mut self.commands,
            &mut self.meshes,
            &self.world,
            self.render_assets.as_deref(),
            &mut self.structure_state,
            &mut self.block_index,
            &mut self.scene_chunks,
            changed,
        );
    }
}

/// 已加载玩法世界、模拟状态与渲染索引的 ECS 访问包
#[derive(SystemParam)]
pub struct PlayingWorldParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub world: ResMut<'w, WorldBlocks>,
    pub render_assets: Option<Res<'w, WorldRenderAssets>>,
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
    /// 调试批量编辑后维护逻辑结构，并在可用时刷新场景
    pub fn refresh_edit_changes(&mut self, changed: &HashSet<IVec3>) {
        refresh_edit_world(
            &mut self.commands,
            &mut self.meshes,
            &self.world,
            self.render_assets.as_deref(),
            &mut self.structure_state,
            &mut self.block_index,
            &mut self.scene_chunks,
            changed,
        );
    }

    /// 世界替换时统一重置模拟历史、待执行操作和派生索引
    pub fn reset_simulation_state(&mut self) {
        self.structure_state.clear();
        self.movement_history.clear();
        self.pusher_state.clear();
        self.pending_effects.clear();
        *self.signal_cache = SignalNetworkCache::default();
    }

    /// 有渲染资源时拆掉并重建场景
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
        rebuild_world(
            &mut self.commands,
            &mut self.meshes,
            &self.world,
            &render_assets,
            &mut self.block_index,
            &mut self.scene_chunks,
        );
    }
}

/// 复用编辑访问包与调试访问包的结构和场景刷新逻辑
fn refresh_edit_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    render_assets: Option<&WorldRenderAssets>,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
    changed: &HashSet<IVec3>,
) {
    structure_state.apply_factory_edit(world, changed);
    let Some(render_assets) = render_assets else {
        return;
    };
    let mut scene = SceneRenderMut {
        commands,
        meshes,
        render_assets,
        block_index,
        scene_chunks,
    };
    refresh_edit_changes(&mut scene, world, changed);
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

    /// 缺少 GPU 渲染资源时，编辑刷新仍先维护结构派生状态
    #[test]
    fn edit_refresh_updates_structures_without_render_assets() {
        let mut ecs = World::new();
        ecs.init_resource::<Assets<Mesh>>();
        ecs.init_resource::<WorldBlocks>();
        ecs.init_resource::<StructureState>();
        ecs.init_resource::<MovementHistory>();
        ecs.init_resource::<PusherState>();
        ecs.init_resource::<PendingTurnEffects>();
        ecs.init_resource::<SignalNetworkCache>();
        ecs.init_resource::<BlockEntityIndex>();
        ecs.init_resource::<SceneChunkMeshes>();
        let mut access = SystemState::<EditableWorldParams>::new(&mut ecs);
        {
            let mut playing = access.get_mut(&mut ecs).unwrap();
            playing.world.insert(
                IVec3::ZERO,
                BlockData::new(BlockKind::Platform, Facing::North),
            );
            playing.structure_state.rebuild_for_runtime(&playing.world);
            playing
                .world
                .insert(IVec3::X, BlockData::new(BlockKind::Platform, Facing::North));
            playing.refresh_edit_changes(&HashSet::from([IVec3::X]));

            let first = playing.structure_state.id_at(IVec3::ZERO);
            assert!(first.is_some());
            assert_eq!(playing.structure_state.id_at(IVec3::X), first);
        }
        access.apply(&mut ecs);
    }
}
