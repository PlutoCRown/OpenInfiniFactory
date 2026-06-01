use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::SolutionState;
use crate::game::systems::world_flow::{primary_click, WorldMenuParams};
use crate::game::ui::{BlockEditAction, OpenBlockPanelDropdown, UiRuntime};
use crate::game::world::rendering::{despawn_world, rebuild_world_for_debug_state};

pub fn block_edit_actions(
    mut click: On<Pointer<Click>>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut solution_state: ResMut<SolutionState>,
    mut world_menu: WorldMenuParams,
    actions: Query<&BlockEditAction>,
) {
    if !primary_click(&mut click) {
        return;
    }
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    let Some(block) = world_menu.world.system_blocks.get(&pos).copied() else {
        ui_runtime.close_current();
        return;
    };
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    block.kind.handle_edit_action(
        pos,
        action,
        &mut world_menu.world,
        &mut solution_state,
        &mut open_dropdown,
    );
    if !block_edit_action_mutates_world(action) {
        return;
    }
    despawn_world(&mut world_menu.commands, &world_menu.block_entities);
    world_menu.factory_structures.clear();
    world_menu.movement_influence.clear();
    world_menu.pusher_state.clear();
    world_menu
        .factory_structures
        .ensure_current_world(&world_menu.world);
    rebuild_world_for_debug_state(
        &mut world_menu.commands,
        &mut world_menu.meshes,
        &world_menu.world,
        &world_menu.render_manager,
        &world_menu.debug,
        &mut world_menu.factory_structures,
    );
}

fn block_edit_action_mutates_world(action: BlockEditAction) -> bool {
    !matches!(
        action,
        BlockEditAction::ToggleMaterialDropdown
            | BlockEditAction::ToggleColorDropdown
            | BlockEditAction::ToggleInputDropdown
            | BlockEditAction::ToggleOutputDropdown
    )
}
