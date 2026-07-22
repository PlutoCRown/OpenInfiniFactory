//! 暂停/背包/快捷栏输入

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::game::state::{GameMode, PlacementState, PlayingUiState, SimulationState};
use crate::game::ui::UiHost;
use crate::game::ui::core::confirm_dialog::ConfirmDialogState;
use crate::game::ui::dismiss_playing_overlay;
use crate::game::ui::{
    CarriedItem, HOTBAR_SLOTS, InlineTextEditState, OpenBlockPanelDropdown, OpenSettingsDropdown,
    PanelDragState, PendingKeyBind, TextPromptState, UiRuntime,
};

/// 关闭覆盖层所需的 UI 资源集合
#[derive(SystemParam)]
pub struct PanelCloseDeps<'w> {
    ui_runtime: ResMut<'w, UiRuntime>,
    ui_host: ResMut<'w, UiHost>,
    confirm: ResMut<'w, ConfirmDialogState>,
    text_prompt: ResMut<'w, TextPromptState>,
    open_block_dropdown: ResMut<'w, OpenBlockPanelDropdown>,
    open_settings_dropdown: ResMut<'w, OpenSettingsDropdown>,
    pending_key_bind: ResMut<'w, PendingKeyBind>,
    inline_edit: ResMut<'w, InlineTextEditState>,
    drag: ResMut<'w, PanelDragState>,
}

impl PanelCloseDeps<'_> {
    /// 尝试关闭当前覆盖层，成功则返回 true
    fn dismiss_overlay(
        &mut self,
        playing_ui: &mut PlayingUiState,
        carried: &mut CarriedItem,
        commands: &mut Commands,
    ) -> bool {
        dismiss_playing_overlay(
            playing_ui,
            carried,
            &mut self.ui_runtime,
            &mut self.ui_host,
            &mut self.confirm,
            &mut self.text_prompt,
            &mut self.open_block_dropdown,
            &mut self.open_settings_dropdown,
            &mut self.pending_key_bind,
            &mut self.inline_edit,
            &mut self.drag,
            commands,
        )
    }
}

/// 处理暂停、背包与快捷栏切换输入
pub fn gameplay_input(
    input: Res<crate::game::input::GameplayInputState>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    text_prompt: Res<TextPromptState>,
    mode: Res<State<GameMode>>,
    mut playing_ui: ResMut<PlayingUiState>,
    mut placement: ResMut<PlacementState>,
    mut carried: ResMut<CarriedItem>,
    mut panel_close: PanelCloseDeps,
    mut simulation: ResMut<SimulationState>,
    mut commands: Commands,
) {
    let typing = panel_close.pending_key_bind.0.is_some()
        || text_prompt.is_open()
        || panel_close.inline_edit.is_active();
    if typing {
        mouse_wheel.clear();
        return;
    }

    if *mode.get() != GameMode::Playing {
        mouse_wheel.clear();
        return;
    }

    if input.pause {
        if panel_close.dismiss_overlay(&mut playing_ui, &mut carried, &mut commands) {
            // Overlay dismissed.
        } else {
            playing_ui.paused = !playing_ui.paused;
            if playing_ui.paused {
                simulation.running = false;
                simulation.step_requested = false;
                simulation.speed = 1.0;
            }
        }
    }

    if input.inventory {
        if panel_close.dismiss_overlay(&mut playing_ui, &mut carried, &mut commands) {
            // Overlay dismissed.
        } else {
            playing_ui.inventory_open = true;
        }
    }

    if panel_close.ui_runtime.blocks_gameplay() || !playing_ui.active_play() {
        mouse_wheel.clear();
        return;
    }

    for (key, index) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ] {
        if keys.just_pressed(key) && index < HOTBAR_SLOTS {
            if placement.selected != index {
                placement.selection.clear();
                placement.edit_gesture = None;
                placement.selected = index;
            }
        }
    }

    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();
    if wheel_delta.abs() > f32::EPSILON {
        let direction = if wheel_delta > 0.0 { -1 } else { 1 };
        let selected = (placement.selected as i32 + direction).rem_euclid(HOTBAR_SLOTS as i32);
        if placement.selected != selected as usize {
            placement.selection.clear();
            placement.edit_gesture = None;
            placement.selected = selected as usize;
        }
    }
}
