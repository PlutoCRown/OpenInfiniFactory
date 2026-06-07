use bevy::prelude::*;

use super::layout::transparent_node;

const CLOSE_ICON_COLOR: Color = Color::WHITE;

pub fn spawn_close_icon(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(transparent_node(Node {
            width: Val::Px(10.0),
            height: Val::Px(10.0),
            position_type: PositionType::Relative,
            ..default()
        }))
        .with_children(|icon| {
            icon.spawn((close_icon_line(45.0), Pickable::IGNORE));
            icon.spawn((close_icon_line(-45.0), Pickable::IGNORE));
        });
}

fn close_icon_line(degrees: f32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(1.0),
            top: Val::Px(4.0),
            width: Val::Px(8.0),
            height: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(CLOSE_ICON_COLOR),
        UiTransform::from_rotation(Rot2::degrees(degrees)),
    )
}
