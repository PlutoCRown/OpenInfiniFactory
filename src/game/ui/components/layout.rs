use bevy::prelude::*;

use super::text::default_button_size;

pub fn transparent_node(style: Node) -> impl Bundle {
    (style, BackgroundColor(Color::NONE), Visibility::Visible)
}

pub fn root_node() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    })
}

pub fn flex_row(height: f32, column_gap: f32) -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        height: Val::Px(default_button_size(height)),
        display: Display::Flex,
        align_items: AlignItems::Center,
        column_gap: Val::Px(column_gap),
        ..default()
    })
}
