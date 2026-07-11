use bevy::prelude::*;

use crate::game::simulation::structure_state::StructureState;
use crate::game::state::{GameMode, PlayingUiState, SimulationState, SolutionState};
use crate::game::systems::debug::DebugState;
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::ui::core::text_input::InlineTextEditState;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::WorldRenderAssets;
use crate::scene::{refresh_edit_changes, BlockEntityIndex};
use crate::shared::config::{ActionKeyName, GameConfig};

use super::EditHistory;

/// 处理 Undo / Redo 快捷键并刷新受影响的方块
pub fn edit_history_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    simulation: Res<SimulationState>,
    ui_runtime: Res<UiRuntime>,
    inline_edit: Res<InlineTextEditState>,
    mut edit_history: ResMut<EditHistory>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut block_index: ResMut<BlockEntityIndex>,
    mut structure_state: ResMut<StructureState>,
    render_assets: Option<Res<WorldRenderAssets>>,
    debug: Res<DebugState>,
) {
    if *mode.get() != GameMode::Playing
        || !playing_ui.active_play()
        || simulation.is_active()
        || ui_runtime.blocks_gameplay()
        || inline_edit.is_active()
    {
        return;
    }

    let patch = if config.chord(ActionKeyName::Redo).just_triggered(&keys) {
        edit_history.redo(&mut world)
    } else if config.chord(ActionKeyName::Undo).just_triggered(&keys) {
        edit_history.undo(&mut world)
    } else {
        return;
    };

    let Some(render_assets) = render_assets.as_ref() else {
        return;
    };
    let Some(patch) = patch else {
        return;
    };

    if patch.touches_goal_or_generator() {
        refresh_static_generated_markers(&mut world);
    }
    refresh_edit_changes(
        &mut commands,
        &mut meshes,
        &mut block_index,
        &world,
        render_assets,
        &debug,
        &mut structure_state,
        &patch.affected_positions(),
    );
    solution_state.dirty = true;
}
