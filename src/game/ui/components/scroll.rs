use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

#[derive(Component)]
#[require(Visibility)]
pub struct ScrollContainer {
    pub offset: f32,
    pub max_offset: f32,
}

#[derive(Component)]
#[require(Visibility)]
pub struct ScrollContent;

pub fn scroll_container(height: f32) -> (impl Bundle, ScrollContainer) {
    (
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(height),
                position_type: PositionType::Relative,
                overflow: Overflow::clip_y(),
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

pub fn scroll_content() -> ScrollContent {
    ScrollContent
}

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
        style.top = Val::Px(-container.offset);
    }
}
