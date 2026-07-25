use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

/// 滚动容器状态
#[derive(Component)]
pub struct ScrollContainer {
    pub offset: f32,
    pub max_offset: f32,
}

/// 滚动内容标记（通过改 top 实现滚动）
#[derive(Component)]
pub struct ScrollContent;

/// 占满父级剩余高度的滚动容器（设置三 Tab 等）
pub fn scroll_container() -> (impl Bundle, ScrollContainer) {
    (
        (
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                overflow: Overflow::clip_y(),
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

/// 滚轮驱动滚动偏移；fill 容器以实际布局高度为视口
pub fn update_scroll_containers(
    mut mouse_wheel: MessageReader<MouseWheel>,
    ui_scale: Res<UiScale>,
    mut containers: Query<(&mut ScrollContainer, &Children, &ComputedNode), Without<ScrollContent>>,
    mut contents: Query<
        (&mut Node, &ComputedNode),
        (With<ScrollContent>, Without<ScrollContainer>),
    >,
) {
    if containers.is_empty() {
        mouse_wheel.clear();
        return;
    }
    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();
    let scale = ui_scale.0.max(0.01);

    for (mut container, children, node) in &mut containers {
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
        // 视口高度：用容器自己 layout 后的高度，避免写死 chrome 比面板还高
        let height_ui = (node.size().y * node.inverse_scale_factor() / scale).max(1.0);

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
    container.offset = (container.offset - drag.delta.y / scale).clamp(0.0, container.max_offset);
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
