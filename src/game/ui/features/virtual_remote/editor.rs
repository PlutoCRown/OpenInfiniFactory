//! 虚拟遥感布局编辑界面

use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Drag, Pointer, Press, Release};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::ui::components::{localized_text, raised_border, ui_logical_bounds, BUTTON_BG};
use crate::game::ui::core::host::{PlayingUiRootEntity, UiRootEntity};
use crate::shared::config::{
    save_config, GameConfig, VirtualControlAnchor, VirtualControlId, VirtualControlsLayout,
};
use crate::shared::touch_profile::TouchProfile;

use super::spawn::{
    apply_knob_node, apply_layout_to_node, control_pixel_size, layout_height_unit,
    spawn_virtual_remote, window_short_edge, CTRL_BG,
};
use super::{VirtualJoystickKnob, VirtualLayoutPreview, VirtualRemoteControl};

/// 布局编辑层叠在设置/主菜单之上
const EDITOR_Z: i32 = 50_000;
const SCALE_MIN: f32 = 0.4;
const SCALE_MAX: f32 = 2.5;

/// 布局编辑器是否打开
#[derive(Resource, Default)]
pub struct VirtualLayoutEditorOpen(pub bool);

/// 布局编辑选中与拖动状态
#[derive(Resource, Default)]
pub struct VirtualLayoutEditorState {
    pub selected: Option<VirtualControlId>,
    pub drag_last: Option<Vec2>,
}

#[derive(Component)]
pub struct VirtualLayoutEditorRoot;

#[derive(Component)]
pub struct VirtualLayoutExitButton;

/// 重置全部遥感布局为默认
#[derive(Component)]
pub struct VirtualLayoutResetButton;

/// 点空白取消选中的全屏底板
#[derive(Component)]
pub struct VirtualLayoutDeselectZone;

/// 顶栏（退出 + 缩放滑条），需压过预览控件
#[derive(Component)]
pub struct VirtualLayoutChrome;

/// 缩放滑条轨道
#[derive(Component)]
pub struct VirtualLayoutScaleSlider;

/// 缩放滑条填充
#[derive(Component)]
pub struct VirtualLayoutScaleFill;

/// 打开遥感布局编辑
pub fn open_virtual_layout_editor(world: &mut World) {
    let enabled = world.resource::<TouchProfile>().enabled;
    if !enabled {
        return;
    }
    world.resource_mut::<VirtualLayoutEditorOpen>().0 = true;
    world.resource_mut::<VirtualLayoutEditorState>().selected = None;
    world.resource_mut::<VirtualLayoutEditorState>().drag_last = None;

    let already = world
        .query_filtered::<Entity, With<VirtualLayoutEditorRoot>>()
        .iter(world)
        .next()
        .is_some();
    if already {
        return;
    }

    // 优先挂在当前 UI 根下（设置从主菜单开时用 UiRoot；游玩中用 PlayingUiRoot）
    let parent = world
        .get_resource::<PlayingUiRootEntity>()
        .map(|r| r.0)
        .or_else(|| world.get_resource::<UiRootEntity>().map(|r| r.0));

    let mut commands = world.commands();
    let spawn_editor = |parent: &mut ChildSpawnerCommands| {
        parent
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.82)),
                VirtualLayoutEditorRoot,
                GlobalZIndex(EDITOR_Z),
                Pickable::IGNORE,
            ))
            .with_children(|chrome| {
                spawn_editor_layers(chrome);
            });
    };

    if let Some(parent) = parent {
        commands.entity(parent).with_children(spawn_editor);
    } else {
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.82)),
                VirtualLayoutEditorRoot,
                GlobalZIndex(EDITOR_Z),
                Pickable::IGNORE,
            ))
            .with_children(|chrome| {
                spawn_editor_layers(chrome);
            });
    }
}

fn spawn_editor_layers(parent: &mut ChildSpawnerCommands) {
    // 空白区：点按取消选中（滑条/退出/控件在更高层）
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        VirtualLayoutDeselectZone,
        Pickable::default(),
    ));

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            VirtualLayoutPreview,
            Pickable::IGNORE,
        ))
        .with_children(|preview| {
            spawn_virtual_remote(preview, &TouchProfile { enabled: true }, true);
        });

    spawn_editor_chrome(parent);

    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(72.0),
            right: Val::Px(16.0),
            max_width: Val::Px(420.0),
            ..default()
        },
        localized_text("virtual.layout_hint", 16.0, Color::srgb(0.9, 0.92, 0.94)),
        Pickable::IGNORE,
        GlobalZIndex(EDITOR_Z + 20),
    ));
}

fn spawn_editor_chrome(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.14, 0.16, 0.92)),
            raised_border(),
            VirtualLayoutChrome,
            GlobalZIndex(EDITOR_Z + 20),
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_BG),
                raised_border(),
                Button,
                VirtualLayoutExitButton,
                Pickable::default(),
            ))
            .with_children(|btn| {
                btn.spawn((
                    localized_text("virtual.layout_exit", 14.0, Color::WHITE),
                    Pickable::IGNORE,
                ));
            });

            bar.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_BG),
                raised_border(),
                Button,
                VirtualLayoutResetButton,
                Pickable::default(),
            ))
            .with_children(|btn| {
                btn.spawn((
                    localized_text("virtual.layout_reset", 14.0, Color::WHITE),
                    Pickable::IGNORE,
                ));
            });

            bar.spawn((
                localized_text("virtual.layout_scale", 14.0, Color::srgb(0.88, 0.9, 0.92)),
                Pickable::IGNORE,
            ));

            bar.spawn((
                Node {
                    width: Val::Px(320.0),
                    height: Val::Px(28.0),
                    padding: UiRect::all(Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.22, 0.24, 0.26)),
                BorderColor::all(Color::srgb(0.4, 0.42, 0.45)),
                VirtualLayoutScaleSlider,
                Pickable::default(),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.32, 0.62, 0.72)),
                    VirtualLayoutScaleFill,
                    Pickable::IGNORE,
                ));
            });
        });
}

/// 退出布局编辑并保存
pub fn exit_virtual_layout_editor(world: &mut World) {
    world.resource_mut::<VirtualLayoutEditorOpen>().0 = false;
    world.resource_mut::<VirtualLayoutEditorState>().selected = None;
    world.resource_mut::<VirtualLayoutEditorState>().drag_last = None;
    let config = world.resource::<GameConfig>().clone();
    save_config(&config);
    let roots: Vec<Entity> = world
        .query_filtered::<Entity, With<VirtualLayoutEditorRoot>>()
        .iter(world)
        .collect();
    for entity in roots {
        if world.get_entity(entity).is_ok() {
            world.entity_mut(entity).despawn();
        }
    }
}

fn scale_to_percent(scale: f32) -> f32 {
    ((scale - SCALE_MIN) / (SCALE_MAX - SCALE_MIN) * 100.0).clamp(0.0, 100.0)
}

fn percent_to_scale(percent: f32) -> f32 {
    SCALE_MIN + percent.clamp(0.0, 1.0) * (SCALE_MAX - SCALE_MIN)
}

pub fn on_editor_control_click(
    mut click: On<Pointer<Click>>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    mut editor: ResMut<VirtualLayoutEditorState>,
    controls: Query<&VirtualRemoteControl>,
    exit_buttons: Query<(), With<VirtualLayoutExitButton>>,
    reset_buttons: Query<(), With<VirtualLayoutResetButton>>,
    deselect_zones: Query<(), With<VirtualLayoutDeselectZone>>,
    scale_sliders: Query<(), With<VirtualLayoutScaleSlider>>,
    mut commands: Commands,
) {
    if !editor_open.0 || click.event.button != PointerButton::Primary {
        return;
    }
    if exit_buttons.get(click.entity).is_ok() {
        click.propagate(false);
        commands.queue(|world: &mut World| {
            exit_virtual_layout_editor(world);
        });
        return;
    }
    if reset_buttons.get(click.entity).is_ok() {
        click.propagate(false);
        commands.queue(|world: &mut World| {
            world.resource_mut::<GameConfig>().virtual_controls = VirtualControlsLayout::DEFAULT;
            world.resource_mut::<VirtualLayoutEditorState>().selected = None;
            world.resource_mut::<VirtualLayoutEditorState>().drag_last = None;
            let config = world.resource::<GameConfig>().clone();
            save_config(&config);
        });
        return;
    }
    if scale_sliders.get(click.entity).is_ok() {
        click.propagate(false);
        return;
    }
    if deselect_zones.get(click.entity).is_ok() {
        click.propagate(false);
        editor.selected = None;
        editor.drag_last = None;
        return;
    }
    let Ok(control) = controls.get(click.entity) else {
        return;
    };
    click.propagate(false);
    editor.selected = Some(control.0);
    editor.drag_last = None;
}

pub fn on_editor_drag(
    mut drag: On<Pointer<Drag>>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    mut editor: ResMut<VirtualLayoutEditorState>,
    mut config: ResMut<GameConfig>,
    windows: Query<&Window, With<PrimaryWindow>>,
    controls: Query<&VirtualRemoteControl>,
    scale_sliders: Query<(&ComputedNode, &UiGlobalTransform), With<VirtualLayoutScaleSlider>>,
) {
    if !editor_open.0 || drag.event.button != PointerButton::Primary {
        return;
    }

    // 缩放滑条：按轨道横向位置改选中控件大小，不取消选中
    if let Ok((node, transform)) = scale_sliders.get(drag.entity) {
        drag.propagate(false);
        let Some(selected) = editor.selected else {
            return;
        };
        let bounds = ui_logical_bounds(node, transform);
        if bounds.width() <= 1.0 {
            return;
        }
        let percent =
            ((drag.pointer_location.position.x - bounds.min.x) / bounds.width()).clamp(0.0, 1.0);
        let mut t = config.virtual_controls.transform(selected);
        t.scale = percent_to_scale(percent);
        config.virtual_controls.set_transform(selected, t);
        return;
    }

    // 只拖当前指下的控件；换控件时切换选中
    let Ok(control) = controls.get(drag.entity) else {
        return;
    };
    drag.propagate(false);
    let pos = drag.pointer_location.position;
    if editor.selected != Some(control.0) {
        editor.selected = Some(control.0);
        editor.drag_last = Some(pos);
        return;
    }
    let Some(last) = editor.drag_last else {
        editor.drag_last = Some(pos);
        return;
    };
    let delta = pos - last;
    editor.drag_last = Some(pos);

    let height_unit = windows
        .single()
        .map(|w| layout_height_unit(window_short_edge(w)))
        .unwrap_or(1.0);
    let dx = delta.x / height_unit;
    let dy = delta.y / height_unit;

    let mut transform = config.virtual_controls.transform(control.0);
    match control.0.anchor() {
        VirtualControlAnchor::BottomLeft => {
            transform.offset_x = (transform.offset_x + dx).max(0.0);
            transform.offset_y = (transform.offset_y - dy).max(0.0);
        }
        VirtualControlAnchor::BottomRight => {
            transform.offset_x = (transform.offset_x - dx).max(0.0);
            transform.offset_y = (transform.offset_y - dy).max(0.0);
        }
        VirtualControlAnchor::TopRight | VirtualControlAnchor::TopRightColumn => {
            transform.offset_x = (transform.offset_x - dx).max(0.0);
            transform.offset_y = (transform.offset_y + dy).max(0.0);
        }
        VirtualControlAnchor::BottomCenter => {
            transform.offset_x += dx;
            transform.offset_y = (transform.offset_y - dy).max(0.0);
        }
    }
    config.virtual_controls.set_transform(control.0, transform);
}

pub fn on_editor_release(
    release: On<Pointer<Release>>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    mut editor: ResMut<VirtualLayoutEditorState>,
) {
    if !editor_open.0 || release.event.button != PointerButton::Primary {
        return;
    }
    editor.drag_last = None;
}

pub fn on_editor_scale_press(
    mut press: On<Pointer<Press>>,
    editor_open: Res<VirtualLayoutEditorOpen>,
    editor: Res<VirtualLayoutEditorState>,
    mut config: ResMut<GameConfig>,
    scale_sliders: Query<(&ComputedNode, &UiGlobalTransform), With<VirtualLayoutScaleSlider>>,
) {
    if !editor_open.0 || press.event.button != PointerButton::Primary {
        return;
    }
    let Ok((node, transform)) = scale_sliders.get(press.entity) else {
        return;
    };
    press.propagate(false);
    let Some(selected) = editor.selected else {
        return;
    };
    let bounds = ui_logical_bounds(node, transform);
    if bounds.width() <= 1.0 {
        return;
    }
    let percent =
        ((press.pointer_location.position.x - bounds.min.x) / bounds.width()).clamp(0.0, 1.0);
    let mut t = config.virtual_controls.transform(selected);
    t.scale = percent_to_scale(percent);
    config.virtual_controls.set_transform(selected, t);
}

pub fn update_layout_editor_ui(
    editor_open: Res<VirtualLayoutEditorOpen>,
    editor: Res<VirtualLayoutEditorState>,
    touch: Res<TouchProfile>,
    config: Res<GameConfig>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut controls: Query<
        (
            Entity,
            &VirtualRemoteControl,
            &mut Node,
            &mut BackgroundColor,
            &mut Visibility,
            &mut GlobalZIndex,
            Option<&Children>,
        ),
        With<VirtualLayoutPreview>,
    >,
    mut knobs: Query<
        &mut Node,
        (
            With<VirtualJoystickKnob>,
            Without<VirtualRemoteControl>,
            Without<VirtualLayoutScaleFill>,
        ),
    >,
    mut fills: Query<
        &mut Node,
        (
            With<VirtualLayoutScaleFill>,
            Without<VirtualRemoteControl>,
            Without<VirtualJoystickKnob>,
        ),
    >,
) {
    if !touch.enabled || !editor_open.0 {
        return;
    }
    let height_unit = windows
        .single()
        .map(|w| layout_height_unit(window_short_edge(w)))
        .unwrap_or(1.0);
    for (_entity, control, mut node, mut bg, mut visibility, mut z, children) in &mut controls {
        *visibility = Visibility::Visible;
        *z = GlobalZIndex(EDITOR_Z + 1);
        let transform = config.virtual_controls.transform(control.0);
        apply_layout_to_node(control.0, transform, height_unit, &mut node);
        *bg = if editor.selected == Some(control.0) {
            Color::srgba(0.55, 0.72, 0.95, 0.75).into()
        } else {
            CTRL_BG.into()
        };
        if control.0 == VirtualControlId::Joystick {
            let size = control_pixel_size(control.0, transform, height_unit);
            if let Some(children) = children {
                for child in children.iter() {
                    if let Ok(mut knob) = knobs.get_mut(child) {
                        apply_knob_node(&mut knob, size, Vec2::ZERO);
                    }
                }
            }
        }
    }

    let fill_percent = editor
        .selected
        .map(|id| scale_to_percent(config.virtual_controls.transform(id).scale))
        .unwrap_or(0.0);
    for mut fill in &mut fills {
        fill.width = Val::Percent(fill_percent);
    }
}
