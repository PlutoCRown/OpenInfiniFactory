use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// 滚动容器状态
#[derive(Component)]
pub struct ScrollContainer {
    pub offset: f32,
    pub max_offset: f32,
    /// 从窗口高度扣除的标题/Tab/内边距等，用于把容器限制在屏内
    pub window_chrome: f32,
}

/// 滚动内容标记（通过改 top 实现滚动）
#[derive(Component)]
pub struct ScrollContent;

/// 高度取 min(内容, 窗口 − chrome)，避免设置面板撑破屏幕
pub fn scroll_container(window_chrome: f32) -> (impl Bundle, ScrollContainer) {
    (
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(200.0),
                position_type: PositionType::Relative,
                overflow: Overflow::clip_y(),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::NONE),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
        ),
        ScrollContainer {
            offset: 0.0,
            max_offset: 0.0,
            window_chrome,
        },
    )
}

/// 滚动内容列
pub fn scroll_content() -> impl Bundle {
    (
        ScrollContent,
        Node {
            width: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            flex_shrink: 0.0,
            // Bevy 0.19.0：Visible 祖先会截断 clip_check，必须非 Visible 才能走到容器裁剪
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::NONE),
    )
}

/// 滚轮驱动滚动偏移，并按窗口剩余高度收紧容器
pub fn update_scroll_containers(
    mut mouse_wheel: MessageReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut containers: Query<(&mut ScrollContainer, &mut Node, &Children, &ComputedNode)>,
    mut contents: Query<(&mut Node, &ComputedNode), With<ScrollContent>>,
) {
    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();
    let scale = ui_scale.0.max(0.01);
    let window_h = windows
        .single()
        .map(|window| window.height())
        .unwrap_or(720.0);
    let available_ui = (window_h / scale).max(80.0);

    for (mut container, mut style, children, node) in &mut containers {
        if node.is_empty() {
            continue;
        }
        let Some(child) = children.iter().find(|child| contents.get(*child).is_ok()) else {
            continue;
        };
        let Ok((mut content_style, content_node)) = contents.get_mut(child) else {
            continue;
        };

        let inv = content_node.inverse_scale_factor();
        let content_ui = content_node.size().y * inv / scale;
        let max_ui = (available_ui - container.window_chrome).max(80.0);
        let height_ui = if content_ui <= 1.0 {
            max_ui
        } else {
            content_ui.min(max_ui)
        };
        let next_height = Val::Px(height_ui);
        if style.height != next_height {
            style.height = next_height;
        }

        container.max_offset = (content_ui - height_ui).max(0.0);
        if wheel_delta.abs() > f32::EPSILON {
            container.offset =
                (container.offset - wheel_delta * 32.0).clamp(0.0, container.max_offset);
        } else {
            container.offset = container.offset.clamp(0.0, container.max_offset);
        }
        let next_top = Val::Px(-container.offset);
        if content_style.top != next_top {
            content_style.top = next_top;
        }
    }
}

/// 触控/鼠标拖动滚动（拖滑条时不滚动，避免抢走控件操作）
pub fn scroll_dragged(
    mut drag: On<Pointer<Drag>>,
    mut containers: Query<&mut ScrollContainer>,
    parents: Query<&ChildOf>,
    sliders: Query<(), With<bevy::ui_widgets::Slider>>,
    ui_scale: Res<UiScale>,
    mut contents: Query<&mut Node, With<ScrollContent>>,
    children: Query<&Children>,
) {
    if drag.event.button != PointerButton::Primary {
        return;
    }
    let mut entity = drag.entity;
    let container_entity = loop {
        if sliders.contains(entity) {
            return;
        }
        if containers.contains(entity) {
            break entity;
        }
        let Ok(parent) = parents.get(entity) else {
            return;
        };
        entity = parent.parent();
    };
    let Ok(mut container) = containers.get_mut(container_entity) else {
        return;
    };
    let scale = ui_scale.0.max(0.01);
    container.offset =
        (container.offset - drag.delta.y / scale).clamp(0.0, container.max_offset);
    let Ok(kids) = children.get(container_entity) else {
        return;
    };
    let Some(child) = kids.iter().find(|child| contents.get(*child).is_ok()) else {
        return;
    };
    if let Ok(mut style) = contents.get_mut(child) {
        style.top = Val::Px(-container.offset);
    }
    drag.propagate(false);
}

/// 绕过 Bevy 0.19.0 clip_check_recursive 回归：中间 Visible 祖先会提前返回，滚出视口仍挡上方控件
pub fn fix_scroll_clip_picking(
    contents: Query<Entity, Added<ScrollContent>>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
) {
    for root in &contents {
        let mut stack = vec![root];
        while let Some(entity) = stack.pop() {
            if let Ok(mut node) = nodes.get_mut(entity) {
                if node.overflow.is_visible() {
                    node.overflow = Overflow::clip();
                }
            }
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
        }
    }
}
