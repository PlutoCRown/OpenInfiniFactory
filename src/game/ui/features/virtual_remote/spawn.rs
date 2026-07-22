//! 虚拟遥感控件生成

use bevy::prelude::*;

use crate::game::ui::components::{localized_text, pressed_border, raised_border, text};
use crate::shared::config::{
    VIRTUAL_LAYOUT_REF_EDGE, VirtualControlAnchor, VirtualControlId, VirtualControlTransform,
    VirtualControlsLayout,
};
use crate::shared::touch_profile::TouchProfile;

use super::{
    VirtualBlockConfigButton, VirtualJoystickKnob, VirtualLandscapeOverlay, VirtualLayoutPreview,
    VirtualLookZone, VirtualPlayOnly, VirtualRemoteControl, VirtualRemoteHud, VirtualSimOnly,
};

/// 虚拟遥感 HUD 根节点
#[derive(Component)]
pub struct VirtualRemoteRoot;

const JOYSTICK_BASE: f32 = 120.0;
const ACTION_BTN: f32 = 72.0;
const SMALL_BTN: f32 = 56.0;

/// 控件默认底色
pub const CTRL_BG: Color = Color::srgba(0.35, 0.38, 0.42, 0.55);
/// 控件按下底色
pub const CTRL_BG_PRESSED: Color = Color::srgba(0.18, 0.20, 0.24, 0.82);
/// 摇杆芯默认色
pub const KNOB_BG: Color = Color::srgba(0.75, 0.78, 0.82, 0.75);
/// 摇杆芯按下色
pub const KNOB_BG_PRESSED: Color = Color::srgba(0.55, 0.58, 0.62, 0.95);

/// 屏幕短边 → 参考短边单位（仅指针位移换算用；控件 Node 走 VMin）
pub fn layout_height_unit(short_edge: f32) -> f32 {
    (short_edge / VIRTUAL_LAYOUT_REF_EDGE).max(0.01)
}

/// 取窗口短边（宽高较小值，逻辑像素）
pub fn window_short_edge(window: &Window) -> f32 {
    window.width().min(window.height())
}

/// 参考短边坐标 → 视口短边百分比（Bevy 按 physical viewport 解析，与 DPI 无关）
pub fn ref_to_vmin(ref_px: f32) -> Val {
    Val::VMin(ref_px / VIRTUAL_LAYOUT_REF_EDGE * 100.0)
}

/// 控件在参考短边（720）下的边长
pub fn control_ref_size(id: VirtualControlId, transform: VirtualControlTransform) -> f32 {
    control_base_size(id) * transform.scale.max(0.4)
}

/// 控件逻辑像素边长（指针命中 / 摇杆芯偏移用）
pub fn control_pixel_size(
    id: VirtualControlId,
    transform: VirtualControlTransform,
    height_unit: f32,
) -> f32 {
    control_ref_size(id, transform) * height_unit
}

/// 在 PlayingUiRoot / 布局编辑器下生成虚拟遥感
pub fn spawn_virtual_remote(
    root: &mut ChildSpawnerCommands,
    touch: &TouchProfile,
    for_editor: bool,
) {
    if !touch.enabled {
        return;
    }

    let mut entity = root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        VirtualRemoteRoot,
        VirtualRemoteHud,
        Pickable::IGNORE,
    ));
    if for_editor {
        entity.insert(VirtualLayoutPreview);
    }
    entity.with_children(|hud| {
        hud.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            VirtualLookZone,
            Pickable::default(),
        ));

        spawn_control(hud, VirtualControlId::Joystick, None, for_editor);
        spawn_control(
            hud,
            VirtualControlId::Jump,
            Some("virtual.short.jump"),
            for_editor,
        );
        spawn_control(
            hud,
            VirtualControlId::Place,
            Some("virtual.short.place"),
            for_editor,
        );
        spawn_control(
            hud,
            VirtualControlId::Delete,
            Some("virtual.short.delete"),
            for_editor,
        );
        spawn_control(
            hud,
            VirtualControlId::Pause,
            Some("virtual.short.pause"),
            for_editor,
        );
        spawn_control(
            hud,
            VirtualControlId::Simulate,
            Some("virtual.short.simulate"),
            for_editor,
        );
        spawn_control(
            hud,
            VirtualControlId::SimPause,
            Some("virtual.short.sim_pause"),
            for_editor,
        );
        spawn_control(
            hud,
            VirtualControlId::SimFast,
            Some("virtual.short.sim_fast"),
            for_editor,
        );
        spawn_control(
            hud,
            VirtualControlId::SimStep,
            Some("virtual.short.sim_step"),
            for_editor,
        );
        spawn_control(hud, VirtualControlId::Rotate, Some("R"), for_editor);
        spawn_control(hud, VirtualControlId::Alternate, Some("C"), for_editor);
        spawn_control(hud, VirtualControlId::Inventory, Some("E"), for_editor);
        spawn_control(
            hud,
            VirtualControlId::BlockConfig,
            Some("virtual.short.config"),
            for_editor,
        );

        if !for_editor {
            hud.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.92)),
                VirtualLandscapeOverlay,
                GlobalZIndex(1000),
                Pickable::default(),
            ))
            .with_children(|overlay| {
                overlay.spawn(localized_text(
                    "virtual.landscape_hint",
                    22.0,
                    Color::srgb(0.92, 0.94, 0.96),
                ));
            });
        }
    });
}

fn spawn_control(
    parent: &mut ChildSpawnerCommands,
    id: VirtualControlId,
    label: Option<&'static str>,
    for_editor: bool,
) {
    let transform = VirtualControlsLayout::DEFAULT.transform(id);
    let mut node = Node::default();
    apply_layout_to_node(id, transform, &mut node);

    let mut entity = parent.spawn((
        node,
        BackgroundColor(CTRL_BG),
        raised_border(),
        VirtualRemoteControl(id),
        VirtualRemoteHud,
        GlobalZIndex(10),
    ));
    entity.insert(Pickable::default());
    if for_editor {
        entity.insert(VirtualLayoutPreview);
    }

    match id {
        VirtualControlId::Simulate => {
            entity.insert(VirtualPlayOnly);
        }
        VirtualControlId::SimPause | VirtualControlId::SimFast | VirtualControlId::SimStep => {
            entity.insert(VirtualSimOnly);
        }
        VirtualControlId::BlockConfig => {
            entity.insert(VirtualBlockConfigButton);
            if !for_editor {
                entity.insert(Visibility::Hidden);
            }
        }
        _ => {}
    }

    entity.with_children(|btn| {
        if id == VirtualControlId::Joystick {
            let mut knob = Node::default();
            // 生成时无窗口；offset=0 时 outer 仅作除数占位
            apply_knob_node(&mut knob, control_ref_size(id, transform), Vec2::ZERO);
            btn.spawn((
                knob,
                BackgroundColor(KNOB_BG),
                raised_border(),
                VirtualJoystickKnob,
                Pickable::IGNORE,
            ));
        } else if let Some(key) = label {
            if key.len() == 1 {
                btn.spawn((text(key, 18.0, Color::WHITE), Pickable::IGNORE));
            } else {
                btn.spawn((localized_text(key, 14.0, Color::WHITE), Pickable::IGNORE));
            }
        }
    });
}

fn anchor_node(anchor: VirtualControlAnchor, t: VirtualControlTransform, size_ref: f32) -> Node {
    let mut node = Node {
        width: ref_to_vmin(size_ref),
        height: ref_to_vmin(size_ref),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
        border_radius: BorderRadius::all(Val::Percent(50.0)),
        ..default()
    };
    match anchor {
        VirtualControlAnchor::BottomLeft => {
            node.left = ref_to_vmin(t.offset_x);
            node.bottom = ref_to_vmin(t.offset_y);
        }
        VirtualControlAnchor::BottomRight => {
            node.right = ref_to_vmin(t.offset_x);
            node.bottom = ref_to_vmin(t.offset_y);
        }
        VirtualControlAnchor::TopRight => {
            node.right = ref_to_vmin(t.offset_x);
            node.top = ref_to_vmin(t.offset_y);
        }
        VirtualControlAnchor::TopRightColumn => {
            node.right = ref_to_vmin(t.offset_x);
            node.top = ref_to_vmin(t.offset_y);
        }
        VirtualControlAnchor::BottomCenter => {
            node.left = Val::Percent(50.0);
            node.bottom = ref_to_vmin(t.offset_y);
            node.margin = UiRect {
                left: ref_to_vmin(-size_ref * 0.5 + t.offset_x),
                ..default()
            };
        }
    }
    node
}

/// 按存档变换刷新控件 Node（VMin = 视口短边比例，跨 DPI 占屏一致）
pub fn apply_layout_to_node(
    id: VirtualControlId,
    transform: VirtualControlTransform,
    node: &mut Node,
) {
    let size_ref = control_ref_size(id, transform);
    *node = anchor_node(id.anchor(), transform, size_ref);
    node.position_type = PositionType::Absolute;
}

/// 刷新摇杆芯（相对父盘百分比；stick_offset 为父盘逻辑像素位移）
pub fn apply_knob_node(knob: &mut Node, outer_logical: f32, stick_offset: Vec2) {
    const KNOB_FRAC: f32 = 0.42;
    let base = (0.5 - KNOB_FRAC * 0.5) * 100.0;
    let denom = outer_logical.max(1.0);
    knob.width = Val::Percent(KNOB_FRAC * 100.0);
    knob.height = Val::Percent(KNOB_FRAC * 100.0);
    knob.position_type = PositionType::Absolute;
    knob.left = Val::Percent(base + stick_offset.x / denom * 100.0);
    knob.top = Val::Percent(base + stick_offset.y / denom * 100.0);
    knob.border_radius = BorderRadius::all(Val::Percent(50.0));
    knob.border = UiRect::all(Val::Px(2.0));
}

/// 控件按下外观
pub fn set_control_pressed_style(
    bg: &mut BackgroundColor,
    border: &mut BorderColor,
    pressed: bool,
) {
    if pressed {
        *bg = CTRL_BG_PRESSED.into();
        *border = pressed_border();
    } else {
        *bg = CTRL_BG.into();
        *border = raised_border();
    }
}

/// 摇杆芯按下外观
pub fn set_knob_pressed_style(bg: &mut BackgroundColor, pressed: bool) {
    *bg = if pressed {
        KNOB_BG_PRESSED.into()
    } else {
        KNOB_BG.into()
    };
}

pub fn control_base_size(id: VirtualControlId) -> f32 {
    match id {
        VirtualControlId::Joystick => JOYSTICK_BASE,
        VirtualControlId::Jump | VirtualControlId::Place | VirtualControlId::Delete => ACTION_BTN,
        _ => SMALL_BTN,
    }
}
