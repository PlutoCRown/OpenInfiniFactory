use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::block_editing::{
    BlockPanelAction, OpenBlockPanelDropdown,
};
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::{primary_click, read_inline_text_input, InlineTextEditState};
use crate::game::session::PlayingWorldParams;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{despawn_world, rebuild_world_for_debug_state};

pub fn block_panel_actions(
    mut click: On<Pointer<Click>>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut inline_edit: ResMut<InlineTextEditState>,
    mut solution_state: ResMut<SolutionState>,
    mut world: PlayingWorldParams,
    actions: Query<&BlockPanelAction>,
) {
    if !primary_click(&mut click) {
        return;
    }
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    let Some(block) = world.world.system_blocks.get(&pos).copied() else {
        ui_runtime.close_current();
        return;
    };
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    if action == BlockPanelAction::StartTeleportRename {
        let settings = world.world.teleport_settings(pos);
        inline_edit.start(
            UiPanelId::Teleport,
            pos,
            "teleport_name",
            settings.name,
        );
        return;
    }

    block.kind.handle_edit_action(
        pos,
        action,
        &mut world.world,
        &mut solution_state,
        &mut open_dropdown,
    );

    if !action.mutates_world() {
        return;
    }
    despawn_world(&mut world.commands, &world.block_entities);
    world.factory_structures.clear();
    world.movement_influence.clear();
    world.pusher_state.clear();
    world
        .factory_structures
        .ensure_current_world(&world.world);
    if let Some(render_assets) = world.render_assets.as_deref() {
        rebuild_world_for_debug_state(
            &mut world.commands,
            &mut world.meshes,
            &world.world,
            render_assets,
            &world.debug,
            &mut world.factory_structures,
        );
    }
}

pub fn inline_text_edit_input(
    ui_runtime: Res<UiRuntime>,
    mut inline_edit: ResMut<InlineTextEditState>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut keyboard_input: MessageReader<bevy::input::keyboard::KeyboardInput>,
) {
    if !inline_edit.is_active() {
        return;
    }
    if ui_runtime.active_panel() != inline_edit.panel {
        inline_edit.clear();
        return;
    }

    let pos = inline_edit.pos.expect("active inline edit has pos");
    let field = inline_edit.field.expect("active inline edit has field");
    let result = read_inline_text_input(&mut keyboard_input, &mut inline_edit.buffer);

    if result.confirm {
        if field == "teleport_name" {
            let mut settings = world.teleport_settings(pos);
            let trimmed = inline_edit.buffer.trim();
            if !trimmed.is_empty() {
                settings.name = trimmed.chars().take(24).collect();
                world.set_teleport_settings(pos, settings);
                solution_state.dirty = true;
            }
        }
        inline_edit.clear();
    } else if result.cancel {
        inline_edit.clear();
    }
}
