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

/// 固定高度纵向滚动容器
pub fn scroll_container(height: f32) -> (impl Bundle, ScrollContainer) {
    (
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(height),
                position_type: PositionType::Relative,
                overflow: Overflow::clip_y(),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::NONE),
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
            flex_shrink: 0.0,
            // Bevy 0.19.0：Visible 祖先会截断 clip_check，必须非 Visible 才能走到容器裁剪
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::NONE),
    )
}

/// 滚轮驱动滚动偏移
pub fn update_scroll_containers(
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut containers: Query<(&mut ScrollContainer, &Children, &ComputedNode)>,
    mut contents: Query<(&mut Node, &ComputedNode), With<ScrollContent>>,
) {
    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();

    for (mut container, children, node) in &mut containers {
        let Some(child) = children.iter().find(|child| contents.get(*child).is_ok()) else {
            continue;
        };
        let Ok((mut style, content_node)) = contents.get_mut(child) else {
            continue;
        };

        container.max_offset = (content_node.size().y - node.size().y).max(0.0);
        if wheel_delta.abs() > f32::EPSILON {
            container.offset =
                (container.offset - wheel_delta * 32.0).clamp(0.0, container.max_offset);
        } else {
            container.offset = container.offset.clamp(0.0, container.max_offset);
        }
        let next_top = Val::Px(-container.offset);
        if style.top != next_top {
            style.top = next_top;
        }
    }
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
