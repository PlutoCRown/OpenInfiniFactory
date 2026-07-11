//! 虚拟遥感 HUD：摇杆、动作键、横屏提示、布局编辑

mod editor;
mod spawn;
mod update;

use bevy::prelude::*;

use crate::game::systems::perf::PerfScope;
use crate::shared::config::VirtualControlId;

pub use editor::{
    exit_virtual_layout_editor, open_virtual_layout_editor, VirtualLayoutDraft,
    VirtualLayoutEditorOpen, VirtualLayoutEditorState, EDITOR_Z,
};
pub use spawn::{spawn_virtual_remote, VirtualRemoteRoot};

use update::{
    apply_virtual_control_layout, sync_landscape_overlay, sync_virtual_remote_visibility,
    update_virtual_remote_input,
};

/// 虚拟遥感运行时：手指绑定与边缘状态
#[derive(Resource, Default)]
pub struct VirtualRemoteRuntime {
    pub pointers: Vec<VirtualPointerBinding>,
    pub jump_press_origin: Option<Vec2>,
    pub look_accum: Vec2,
    pub move_accum: Vec2,
    pub sim_fast_held: bool,
    /// 摇杆芯相对盘心的像素偏移（拖动中）
    pub joystick_stick_offset: Vec2,
    /// 当前处于按下外观的控件
    pub pressed_controls: Vec<VirtualControlId>,
    /// 放置/删除按住（按 pointer 捕获，松手任意处才结束）
    pub place_held: bool,
    pub delete_held: bool,
    pub jump_held: bool,
    pub place_just_pressed: bool,
    pub place_just_released: bool,
    pub delete_just_pressed: bool,
    pub delete_just_released: bool,
    pub jump_just_pressed: bool,
}

#[derive(Clone, Debug)]
pub struct VirtualPointerBinding {
    pub pointer_id: bevy::picking::pointer::PointerId,
    pub kind: VirtualPointerKind,
    pub last_pos: Vec2,
    pub origin: Vec2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirtualPointerKind {
    Look,
    Joystick,
    Jump,
    Place,
    Delete,
    SimFast,
    BlockLook,
}

#[derive(Component, Clone, Copy)]
pub struct VirtualRemoteControl(pub VirtualControlId);

#[derive(Component)]
pub struct VirtualLookZone;

#[derive(Component)]
pub struct VirtualJoystickKnob;

#[derive(Component)]
pub struct VirtualLandscapeOverlay;

#[derive(Component)]
pub struct VirtualRemoteHud;

#[derive(Component)]
pub struct VirtualSimOnly;

#[derive(Component)]
pub struct VirtualEditOnly;

#[derive(Component)]
pub struct VirtualBlockConfigButton;

/// 布局编辑器内的预览控件根（与游玩 HUD 分离）
#[derive(Component)]
pub struct VirtualLayoutPreview;

pub struct VirtualRemotePlugin;

impl Plugin for VirtualRemotePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VirtualRemoteRuntime>()
            .init_resource::<VirtualLayoutEditorState>()
            .init_resource::<VirtualLayoutEditorOpen>()
            .init_resource::<VirtualLayoutDraft>()
            .add_observer(update::on_virtual_press)
            .add_observer(update::on_virtual_drag)
            .add_observer(update::on_virtual_release)
            .add_observer(update::on_virtual_click)
            .add_observer(editor::on_editor_control_click)
            .add_observer(editor::on_editor_drag)
            .add_observer(editor::on_editor_release)
            .add_observer(editor::on_editor_scale_press)
            .add_systems(
                Update,
                (
                    update_virtual_remote_input,
                    apply_virtual_control_layout,
                    sync_virtual_remote_visibility,
                    sync_landscape_overlay,
                    editor::update_layout_editor_ui,
                )
                    .chain()
                    .before(crate::game::input::gather_gameplay_input)
                    .before(PerfScope::Input),
            );
    }
}
