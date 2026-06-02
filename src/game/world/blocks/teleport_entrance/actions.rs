use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{SolutionState, TeleportRenameState};
use crate::game::systems::world_flow::{primary_click, push_text_input, toggle_block_dropdown};
use crate::game::ui::{
    BlockPanelDropdown, BlockSettingsChanged, OpenBlockPanelDropdown, TeleportAction, UiPanelKey,
    UiRuntime,
};
use crate::game::world::blocks::{set_teleport_settings, teleport_settings};
use crate::game::world::grid::WorldBlocks;

pub fn teleport_menu_actions(
    mut click: On<Pointer<Click>>,
    ui_runtime: ResMut<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut rename_state: ResMut<TeleportRenameState>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut block_settings_changed: MessageWriter<BlockSettingsChanged>,
    actions: Query<&TeleportAction>,
) {
    if !primary_click(&mut click) || ui_runtime.active_key() != Some(UiPanelKey::TELEPORT) {
        return;
    }

    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    match action {
        TeleportAction::TogglePairDropdown => {
            toggle_block_dropdown(&mut open_dropdown, BlockPanelDropdown::TeleportPair);
        }
        TeleportAction::SetPair(pair) => {
            let mut settings = teleport_settings(&world, pos);
            settings.pair = pair;
            set_teleport_settings(&mut world, pos, settings);
            solution_state.dirty = true;
            block_settings_changed.write(BlockSettingsChanged { pos });
            open_dropdown.0 = None;
        }
        TeleportAction::Rename => {
            let settings = teleport_settings(&world, pos);
            rename_state.editing = Some(pos);
            rename_state.buffer = settings.name;
        }
    }
}

pub fn teleport_rename_input(
    ui_runtime: Res<UiRuntime>,
    mut rename_state: ResMut<TeleportRenameState>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut keyboard_input: MessageReader<KeyboardInput>,
    mut block_settings_changed: MessageWriter<BlockSettingsChanged>,
) {
    if ui_runtime.active_key() != Some(UiPanelKey::TELEPORT) || rename_state.editing.is_none() {
        return;
    }

    let pos = rename_state.editing.expect("checked above");
    let mut confirm = false;
    let mut cancel = false;

    for event in keyboard_input.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Enter => confirm = true,
            Key::Escape => cancel = true,
            Key::Backspace => {
                rename_state.buffer.pop();
            }
            _ => push_text_input(&mut rename_state.buffer, event),
        }
    }

    if confirm {
        let mut settings = teleport_settings(&world, pos);
        let trimmed = rename_state.buffer.trim();
        if !trimmed.is_empty() {
            settings.name = trimmed.chars().take(24).collect();
            set_teleport_settings(&mut world, pos, settings);
            solution_state.dirty = true;
            block_settings_changed.write(BlockSettingsChanged { pos });
        }
        rename_state.editing = None;
    } else if cancel {
        rename_state.editing = None;
    }
}
