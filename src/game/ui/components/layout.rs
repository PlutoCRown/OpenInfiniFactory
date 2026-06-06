use bevy::prelude::*;

use super::text::default_button_size;

pub fn transparent_node(style: Node) -> impl Bundle {
    (style, BackgroundColor(Color::NONE))
}

pub fn root_node() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    })
}

/// Logical-pixel bounds in window space (same convention as Bevy's UI viewport widget).
pub fn ui_logical_bounds(computed: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    let inv = computed.inverse_scale_factor();
    Rect::from_center_size(transform.translation.trunc() * inv, computed.size() * inv)
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

pub fn flex_row_auto(min_height: f32, column_gap: f32) -> impl Bundle {
    transparent_node(Node {
        width: Val::Auto,
        height: Val::Auto,
        min_height: Val::Px(default_button_size(min_height)),
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
        column_gap: Val::Px(column_gap),
        row_gap: Val::Px(column_gap),
        ..default()
    })
}
