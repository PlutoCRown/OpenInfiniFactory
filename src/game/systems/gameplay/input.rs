//! 暂停/背包/快捷栏输入

use crate::game::local_player::LocalPlayerMut;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::game::state::{GameMode, SimulationState, SolutionState};
use crate::game::ui::HOTBAR_SLOTS;
use crate::game::ui::PanelCloseDeps;

/// 处理暂停、背包与快捷栏切换输入
pub fn gameplay_input(
    input: Res<crate::game::input::GameplayInputState>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<State<GameMode>>,
    mut player: LocalPlayerMut,
    mut solution_state: ResMut<SolutionState>,
    mut panel_close: PanelCloseDeps,
    mut simulation: ResMut<SimulationState>,
    mut commands: Commands,
) {
    let typing = panel_close.pending_key_bind.0.is_some()
        || panel_close.text_prompt.is_open()
        || panel_close.inline_edit.is_active();
    if typing {
        mouse_wheel.clear();
        return;
    }

    if *mode.get() != GameMode::Playing {
        mouse_wheel.clear();
        return;
    }

    // 模拟中若背包仍开着则关掉，并禁止再次打开
    if simulation.is_active() && player.playing_ui.inventory_open {
        player.playing_ui.inventory_open = false;
    }

    if input.pause {
        if panel_close.dismiss_playing_overlay(
            &mut player.playing_ui,
            &mut player.carried,
            &mut player.inventory,
            &player.placement,
            &mut solution_state,
            &mut commands,
        ) {
            // Overlay dismissed.
        } else {
            player.playing_ui.paused = !player.playing_ui.paused;
            if player.playing_ui.paused {
                simulation.pause();
            }
        }
    }

    if input.inventory {
        if panel_close.dismiss_playing_overlay(
            &mut player.playing_ui,
            &mut player.carried,
            &mut player.inventory,
            &player.placement,
            &mut solution_state,
            &mut commands,
        ) {
            // Overlay dismissed.
        } else if !simulation.is_active() {
            // 模拟期禁止打开背包
            player.playing_ui.inventory_open = true;
        }
    }

    if panel_close.ui_runtime.blocks_gameplay() || !player.playing_ui.active_play() {
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
            if player.placement.selected != index {
                player.placement.selection.clear();
                player.placement.edit_gesture = None;
                player.placement.selected = index;
            }
        }
    }

    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();
    if wheel_delta.abs() > f32::EPSILON {
        let direction = if wheel_delta > 0.0 { -1 } else { 1 };
        let selected =
            (player.placement.selected as i32 + direction).rem_euclid(HOTBAR_SLOTS as i32);
        if player.placement.selected != selected as usize {
            player.placement.selection.clear();
            player.placement.edit_gesture = None;
            player.placement.selected = selected as usize;
        }
    }
}
