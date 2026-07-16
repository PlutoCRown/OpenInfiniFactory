//! 虚拟遥感触摸输入与显隐

use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Drag, Pointer, Press, Release};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::blocks::BlockPresent;
use crate::game::input::{ActionPulse, GameplayInputState};
use crate::game::state::{BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState};
use crate::game::ui::UiRuntime;
use crate::game::world::grid::WorldBlocks;
use crate::shared::config::{GameConfig, VirtualControlId};
use crate::shared::touch_profile::TouchProfile;

use super::editor::VirtualLayoutEditorOpen;
use super::spawn::{
    apply_knob_node, apply_layout_to_node, control_pixel_size, layout_height_unit,
    set_control_pressed_style, set_knob_pressed_style, window_short_edge,
};
use super::{
    VirtualBlockConfigButton, VirtualJoystickKnob, VirtualLandscapeOverlay, VirtualLayoutPreview,
    VirtualLookZone, VirtualPlayOnly, VirtualPointerBinding, VirtualPointerKind,
    VirtualRemoteControl, VirtualRemoteHud, VirtualRemoteRuntime, VirtualSimOnly,
};

const JOYSTICK_DEADZONE: f32 = 0.18;
const FLY_SWIPE_THRESHOLD: f32 = 24.0;
const LOOK_SENSITIVITY: f32 = 1.0;

pub fn update_virtual_remote_input(
    touch: Res<TouchProfile>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut input: ResMut<GameplayInputState>,
    mut runtime: ResMut<VirtualRemoteRuntime>,
) {
    input.clear_virtual_axes();
    // 放置/删除边沿每帧由本系统独占写入
    input.virtual_place = ActionPulse::default();
    input.virtual_delete = ActionPulse::default();

    if !touch.enabled || editor_open.0 {
        runtime.pointers.clear();
        runtime.place_held = false;
        runtime.delete_held = false;
        runtime.jump_held = false;
        runtime.place_just_pressed = false;
        runtime.place_just_released = false;
        runtime.delete_just_pressed = false;
        runtime.delete_just_released = false;
        runtime.jump_just_pressed = false;
        runtime.jump_press_origin = None;
        runtime.look_accum = Vec2::ZERO;
        runtime.move_accum = Vec2::ZERO;
        runtime.sim_fast_held = false;
        runtime.joystick_stick_offset = Vec2::ZERO;
        runtime.pressed_controls.clear();
        return;
    }

    // 桌面模拟触控：主指针松开一定结束放置/删除（防止 UI Release 丢失导致粘住）
    if runtime.place_held && mouse_buttons.just_released(MouseButton::Left) {
        runtime.place_held = false;
        runtime.place_just_released = true;
        runtime
            .pointers
            .retain(|b| b.kind != VirtualPointerKind::Place);
        clear_pressed(&mut runtime, VirtualControlId::Place);
    }
    if runtime.delete_held && mouse_buttons.just_released(MouseButton::Left) {
        runtime.delete_held = false;
        runtime.delete_just_released = true;
        runtime
            .pointers
            .retain(|b| b.kind != VirtualPointerKind::Delete);
        clear_pressed(&mut runtime, VirtualControlId::Delete);
    }
    if runtime.jump_held && mouse_buttons.just_released(MouseButton::Left) {
        runtime.jump_held = false;
        runtime.jump_press_origin = None;
        runtime
            .pointers
            .retain(|b| b.kind != VirtualPointerKind::Jump);
        clear_pressed(&mut runtime, VirtualControlId::Jump);
    }
    if runtime.sim_fast_held && mouse_buttons.just_released(MouseButton::Left) {
        runtime.sim_fast_held = false;
        runtime
            .pointers
            .retain(|b| b.kind != VirtualPointerKind::SimFast);
        clear_pressed(&mut runtime, VirtualControlId::SimFast);
    }
    if mouse_buttons.just_released(MouseButton::Left)
        && runtime
            .pointers
            .iter()
            .any(|b| b.kind == VirtualPointerKind::Joystick)
    {
        runtime
            .pointers
            .retain(|b| b.kind != VirtualPointerKind::Joystick);
        runtime.move_accum = Vec2::ZERO;
        runtime.joystick_stick_offset = Vec2::ZERO;
        clear_pressed(&mut runtime, VirtualControlId::Joystick);
    }

    let mut fly_up = false;
    let mut fly_down = false;
    for binding in &runtime.pointers {
        if binding.kind == VirtualPointerKind::Jump {
            if let Some(origin) = runtime.jump_press_origin {
                let dy = origin.y - binding.last_pos.y;
                if dy > FLY_SWIPE_THRESHOLD {
                    fly_up = true;
                } else if dy < -FLY_SWIPE_THRESHOLD {
                    fly_down = true;
                }
            }
        }
    }

    input.virtual_move_axis = runtime.move_accum;
    input.virtual_look_delta = runtime.look_accum;
    runtime.look_accum = Vec2::ZERO;

    input.virtual_fly_up = fly_up;
    input.virtual_fly_down = fly_down;

    input.virtual_place = ActionPulse {
        just_pressed: runtime.place_just_pressed,
        pressed: runtime.place_held,
        just_released: runtime.place_just_released,
    };
    input.virtual_delete = ActionPulse {
        just_pressed: runtime.delete_just_pressed,
        pressed: runtime.delete_held,
        just_released: runtime.delete_just_released,
    };
    input.virtual_jump = ActionPulse {
        just_pressed: runtime.jump_just_pressed,
        pressed: runtime.jump_held,
        just_released: false,
    };
    input.virtual_sim_fast = runtime.sim_fast_held;

    runtime.place_just_pressed = false;
    runtime.place_just_released = false;
    runtime.delete_just_pressed = false;
    runtime.delete_just_released = false;
    runtime.jump_just_pressed = false;
}

fn mark_pressed(runtime: &mut VirtualRemoteRuntime, id: VirtualControlId) {
    if !runtime.pressed_controls.contains(&id) {
        runtime.pressed_controls.push(id);
    }
}

fn clear_pressed(runtime: &mut VirtualRemoteRuntime, id: VirtualControlId) {
    runtime.pressed_controls.retain(|c| *c != id);
}

fn end_pointer_binding(
    runtime: &mut VirtualRemoteRuntime,
    pointer_id: bevy::picking::pointer::PointerId,
) {
    let Some(index) = runtime
        .pointers
        .iter()
        .position(|b| b.pointer_id == pointer_id)
    else {
        return;
    };
    let kind = runtime.pointers[index].kind;
    runtime.pointers.remove(index);
    match kind {
        VirtualPointerKind::Place => {
            if runtime.place_held {
                runtime.place_held = false;
                runtime.place_just_released = true;
            }
            clear_pressed(runtime, VirtualControlId::Place);
        }
        VirtualPointerKind::Delete => {
            if runtime.delete_held {
                runtime.delete_held = false;
                runtime.delete_just_released = true;
            }
            clear_pressed(runtime, VirtualControlId::Delete);
        }
        VirtualPointerKind::Jump => {
            runtime.jump_held = false;
            runtime.jump_press_origin = None;
            clear_pressed(runtime, VirtualControlId::Jump);
        }
        VirtualPointerKind::Joystick => {
            runtime.move_accum = Vec2::ZERO;
            runtime.joystick_stick_offset = Vec2::ZERO;
            clear_pressed(runtime, VirtualControlId::Joystick);
        }
        VirtualPointerKind::SimFast => {
            runtime.sim_fast_held = false;
            clear_pressed(runtime, VirtualControlId::SimFast);
        }
        _ => {}
    }
}

pub fn on_virtual_press(
    mut press: On<Pointer<Press>>,
    touch: Res<TouchProfile>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    placement: Res<PlacementState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    controls: Query<&VirtualRemoteControl>,
    look_zones: Query<(), With<VirtualLookZone>>,
    mut runtime: ResMut<VirtualRemoteRuntime>,
    mut input: ResMut<GameplayInputState>,
) {
    if !touch.enabled
        || editor_open.0
        || *mode.get() != GameMode::Playing
        || !playing_ui.active_play()
        || ui_runtime.blocks_gameplay()
        || press.event.button != PointerButton::Primary
    {
        return;
    }

    let pos = press.pointer_location.position;
    let pointer_id = press.pointer_id;

    if let Ok(control) = controls.get(press.entity) {
        press.propagate(false);
        // 同一指针换控件前先结束旧绑定
        end_pointer_binding(&mut runtime, pointer_id);
        mark_pressed(&mut runtime, control.0);
        // 点按类：只亮按下态，等 Release / Click
        match control.0 {
            VirtualControlId::Pause
            | VirtualControlId::Simulate
            | VirtualControlId::SimPause
            | VirtualControlId::SimStep
            | VirtualControlId::Rotate
            | VirtualControlId::Alternate
            | VirtualControlId::Inventory
            | VirtualControlId::BlockConfig => return,
            _ => {}
        }
        let kind = match control.0 {
            VirtualControlId::Joystick => VirtualPointerKind::Joystick,
            VirtualControlId::Jump => {
                runtime.jump_press_origin = Some(pos);
                runtime.jump_held = true;
                runtime.jump_just_pressed = true;
                VirtualPointerKind::Jump
            }
            VirtualControlId::Place => {
                runtime.place_held = true;
                runtime.place_just_pressed = true;
                VirtualPointerKind::Place
            }
            VirtualControlId::Delete => {
                runtime.delete_held = true;
                runtime.delete_just_pressed = true;
                VirtualPointerKind::Delete
            }
            VirtualControlId::SimFast => {
                runtime.sim_fast_held = true;
                VirtualPointerKind::SimFast
            }
            _ => return,
        };
        runtime.pointers.push(VirtualPointerBinding {
            pointer_id,
            kind,
            last_pos: pos,
            origin: pos,
        });
        return;
    }

    if look_zones.get(press.entity).is_ok() {
        press.propagate(false);
        if placement.edit_gesture.is_some() {
            if let Ok(window) = windows.single() {
                if pos.x >= window.width() * 0.5 {
                    input.virtual_cancel_edit = true;
                    // 右半取消：同时松开虚拟放置/删除按住
                    if runtime.place_held {
                        runtime.place_held = false;
                        runtime.place_just_released = true;
                        runtime
                            .pointers
                            .retain(|b| b.kind != VirtualPointerKind::Place);
                    }
                    if runtime.delete_held {
                        runtime.delete_held = false;
                        runtime.delete_just_released = true;
                        runtime
                            .pointers
                            .retain(|b| b.kind != VirtualPointerKind::Delete);
                    }
                    return;
                }
            }
        }
        end_pointer_binding(&mut runtime, pointer_id);
        runtime.pointers.push(VirtualPointerBinding {
            pointer_id,
            kind: VirtualPointerKind::Look,
            last_pos: pos,
            origin: pos,
        });
    }
}

pub fn on_virtual_drag(
    mut drag: On<Pointer<Drag>>,
    touch: Res<TouchProfile>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    config: Res<GameConfig>,
    windows: Query<&Window, With<PrimaryWindow>>,
    controls: Query<&VirtualRemoteControl>,
    parents: Query<&Children>,
    mut knobs: Query<&mut Node, With<VirtualJoystickKnob>>,
    mut runtime: ResMut<VirtualRemoteRuntime>,
) {
    if !touch.enabled || editor_open.0 || drag.event.button != PointerButton::Primary {
        return;
    }
    let pointer_id = drag.pointer_id;
    let pos = drag.pointer_location.position;
    let Some(index) = runtime
        .pointers
        .iter()
        .position(|b| b.pointer_id == pointer_id)
    else {
        return;
    };

    let delta = pos - runtime.pointers[index].last_pos;
    runtime.pointers[index].last_pos = pos;
    let kind = runtime.pointers[index].kind;
    let origin = runtime.pointers[index].origin;

    match kind {
        VirtualPointerKind::Look | VirtualPointerKind::Place | VirtualPointerKind::Delete => {
            runtime.look_accum += delta * LOOK_SENSITIVITY;
            drag.propagate(false);
        }
        VirtualPointerKind::Jump | VirtualPointerKind::BlockLook | VirtualPointerKind::SimFast => {
            drag.propagate(false);
        }
        VirtualPointerKind::Joystick => {
            drag.propagate(false);
            let height_unit = windows
                .single()
                .map(|w| layout_height_unit(window_short_edge(w)))
                .unwrap_or(1.0);
            let transform = config
                .virtual_controls
                .transform(VirtualControlId::Joystick);
            let size = control_pixel_size(VirtualControlId::Joystick, transform, height_unit);
            let radius = size * 0.5;
            let offset = (pos - origin).clamp_length_max(radius);
            let axis = offset / radius;
            let axis = if axis.length() < JOYSTICK_DEADZONE {
                Vec2::ZERO
            } else {
                axis
            };
            runtime.move_accum = Vec2::new(axis.x, -axis.y);
            runtime.joystick_stick_offset = offset;
            if let Ok(children) = parents.get(drag.entity) {
                for child in children.iter() {
                    if let Ok(mut knob) = knobs.get_mut(child) {
                        apply_knob_node(&mut knob, size, offset);
                    }
                }
            } else if let Ok(control) = controls.get(drag.entity) {
                let _ = control;
            }
        }
    }
}

pub fn on_virtual_release(
    mut release: On<Pointer<Release>>,
    touch: Res<TouchProfile>,
    controls: Query<&VirtualRemoteControl>,
    look_zones: Query<(), With<VirtualLookZone>>,
    mut runtime: ResMut<VirtualRemoteRuntime>,
) {
    if !touch.enabled || release.event.button != PointerButton::Primary {
        return;
    }
    if let Ok(control) = controls.get(release.entity) {
        clear_pressed(&mut runtime, control.0);
        if control.0 == VirtualControlId::Joystick {
            runtime.joystick_stick_offset = Vec2::ZERO;
        }
    }
    let pointer_id = release.pointer_id;
    end_pointer_binding(&mut runtime, pointer_id);
    if controls.get(release.entity).is_ok() || look_zones.get(release.entity).is_ok() {
        release.propagate(false);
    }
}

pub fn on_virtual_click(
    mut click: On<Pointer<Click>>,
    touch: Res<TouchProfile>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    simulation: Res<SimulationState>,
    controls: Query<&VirtualRemoteControl>,
    mut input: ResMut<GameplayInputState>,
) {
    if !touch.enabled
        || editor_open.0
        || *mode.get() != GameMode::Playing
        || !playing_ui.active_play()
        || ui_runtime.blocks_gameplay()
        || click.event.button != PointerButton::Primary
    {
        return;
    }
    let Ok(control) = controls.get(click.entity) else {
        return;
    };
    click.propagate(false);
    match control.0 {
        VirtualControlId::Pause => input.virtual_pause = true,
        VirtualControlId::Inventory => input.virtual_inventory = true,
        VirtualControlId::Simulate => input.virtual_simulate = true,
        // 暂停模拟：回滚并退出模拟态（与 R 回滚一致，回到可编辑的非模拟 HUD）
        VirtualControlId::SimPause => input.virtual_rollback = true,
        VirtualControlId::SimStep => input.virtual_sim_step = true,
        VirtualControlId::Rotate => {
            if simulation.is_active() {
                input.virtual_rollback = true;
            } else {
                input.virtual_rotate = true;
            }
        }
        VirtualControlId::Alternate => {
            if simulation.is_active() {
                input.virtual_sim_step = true;
            } else {
                input.virtual_alternate = true;
            }
        }
        VirtualControlId::BlockConfig => input.virtual_open_block_config = true,
        _ => {}
    }
}

pub fn apply_virtual_control_layout(
    touch: Res<TouchProfile>,
    config: Res<GameConfig>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    runtime: Res<VirtualRemoteRuntime>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut controls: Query<(
        &VirtualRemoteControl,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderColor,
        Option<&Children>,
    )>,
    mut knobs: Query<
        (&mut Node, &mut BackgroundColor),
        (With<VirtualJoystickKnob>, Without<VirtualRemoteControl>),
    >,
) {
    if !touch.enabled || editor_open.0 {
        return;
    }
    let height_unit = windows
        .single()
        .map(|w| layout_height_unit(window_short_edge(w)))
        .unwrap_or(1.0);
    for (control, mut node, mut bg, mut border, children) in &mut controls {
        let transform = config.virtual_controls.transform(control.0);
        apply_layout_to_node(control.0, transform, height_unit, &mut node);
        let pressed = runtime.pressed_controls.contains(&control.0);
        set_control_pressed_style(&mut bg, &mut border, pressed);
        if control.0 == VirtualControlId::Joystick {
            let size = control_pixel_size(control.0, transform, height_unit);
            if let Some(children) = children {
                for child in children.iter() {
                    if let Ok((mut knob, mut knob_bg)) = knobs.get_mut(child) {
                        apply_knob_node(&mut knob, size, runtime.joystick_stick_offset);
                        set_knob_pressed_style(&mut knob_bg, pressed);
                    }
                }
            }
        }
    }
}

pub fn sync_virtual_remote_visibility(
    touch: Res<TouchProfile>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    simulation: Res<SimulationState>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    builder_mode: Res<BuilderMode>,
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    mut controls: Query<(
        Entity,
        &VirtualRemoteControl,
        &mut Visibility,
        Option<&VirtualSimOnly>,
        Option<&VirtualPlayOnly>,
        Option<&VirtualBlockConfigButton>,
    )>,
    mut roots: Query<
        (Entity, &mut Visibility),
        (
            With<VirtualRemoteHud>,
            Without<VirtualRemoteControl>,
            Without<VirtualLandscapeOverlay>,
        ),
    >,
    preview_roots: Query<Entity, With<VirtualLayoutPreview>>,
) {
    if !touch.enabled {
        return;
    }

    // 布局编辑：显示编辑器预览控件，隐藏游玩 HUD
    if editor_open.0 {
        for (entity, mut visibility) in &mut roots {
            *visibility = if preview_roots.contains(entity) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        for (entity, _, mut visibility, _, _, _) in &mut controls {
            *visibility = if preview_roots.contains(entity) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        return;
    }

    let show = *mode.get() == GameMode::Playing
        && playing_ui.active_play()
        && !ui_runtime.blocks_gameplay();
    let sim_active = simulation.is_active();
    let play_mode = *builder_mode == BuilderMode::Play;
    let show_config = show
        && *builder_mode == BuilderMode::Edit
        && !sim_active
        && placement
            .target
            .and_then(|t| world.system_blocks.get(&t.pos))
            .and_then(|b| b.kind.ui_panel())
            .is_some();

    let root_vis = if show {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for (_, mut visibility) in &mut roots {
        *visibility = root_vis;
    }

    for (_, control, mut visibility, sim_only, play_only, block_config) in &mut controls {
        let visible = if block_config.is_some() {
            show_config
        } else if sim_only.is_some() {
            show && play_mode && sim_active
        } else if play_only.is_some() {
            show && play_mode && !sim_active
        } else {
            let _ = control;
            show
        };
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn sync_landscape_overlay(
    touch: Res<TouchProfile>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut overlays: Query<(&mut Visibility, &mut Node), With<VirtualLandscapeOverlay>>,
) {
    if !touch.enabled {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    let show_overlay_support = true;
    #[cfg(not(target_arch = "wasm32"))]
    let show_overlay_support = false;

    if !show_overlay_support {
        for (mut visibility, mut node) in &mut overlays {
            *visibility = Visibility::Hidden;
            node.display = Display::None;
        }
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let portrait = window.height() > window.width();
    for (mut visibility, mut node) in &mut overlays {
        if portrait {
            *visibility = Visibility::Visible;
            node.display = Display::Flex;
        } else {
            *visibility = Visibility::Hidden;
            node.display = Display::None;
        }
    }
}
