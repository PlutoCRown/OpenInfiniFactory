use crate::game::local_player::LocalPlayerMut;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::structure_state::StructureState;
use crate::game::state::SolutionState;
use crate::game::systems::debug::DebugState;
use crate::game::systems::gameplay::GameplayPlayGate;
use crate::game::ui::core::text_input::InlineTextEditState;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::SceneChunkMeshes;
use crate::game::world::rendering::WorldRenderAssets;
use crate::scene::{BlockEntityIndex, SceneRenderMut, refresh_edit_changes};
use crate::shared::config::{ActionKeyName, GameConfig};

/// 撤销/重做后刷新世界渲染所需的查询集合
#[derive(SystemParam)]
pub struct EditHistoryApply<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    block_index: ResMut<'w, BlockEntityIndex>,
    scene_chunks: ResMut<'w, SceneChunkMeshes>,
    structure_state: ResMut<'w, StructureState>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
    debug: Res<'w, DebugState>,
}

impl<'w, 's> EditHistoryApply<'w, 's> {
    /// 撤销/重做后增量刷新受影响格的渲染
    fn refresh_edit_changes(
        &mut self,
        world: &WorldBlocks,
        changed: &std::collections::HashSet<IVec3>,
    ) {
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
        refresh_edit_changes(&mut scene, world, changed);
    }
}

/// 处理 Undo / Redo 快捷键并刷新受影响的方块
pub fn edit_history_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    gate: GameplayPlayGate,
    inline_edit: Res<InlineTextEditState>,
    mut player: LocalPlayerMut,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut apply: EditHistoryApply,
) {
    if !gate.allows_world_edit() || inline_edit.is_active() {
        return;
    }

    let patch = if config.chord(ActionKeyName::Redo).just_triggered(&keys) {
        player
            .edit_history
            .redo(&mut world, &mut player.placement.selection)
    } else if config.chord(ActionKeyName::Undo).just_triggered(&keys) {
        player
            .edit_history
            .undo(&mut world, &mut player.placement.selection)
    } else {
        return;
    };

    let Some(patch) = patch else {
        return;
    };

    // 格子变更后重建焊点等虚方块；验收/生成器仍走同一刷新
    if !patch.cells.is_empty() || patch.touches_goal_or_generator() {
        refresh_static_generated_markers(&mut world);
    }
    if apply.render_assets.is_none() {
        return;
    }
    apply.refresh_edit_changes(&world, &patch.affected_positions());
    solution_state.dirty = true;
}
