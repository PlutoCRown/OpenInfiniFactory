use bevy::prelude::*;

use super::components::default_font_size;

pub const PANEL_BG: Color = Color::srgb(0.192, 0.188, 0.192);
pub const PANEL_LIGHT_EDGE: Color = Color::srgb(0.40, 0.38, 0.36);
pub const PANEL_DARK_EDGE: Color = Color::srgb(0.08, 0.06, 0.05);
pub const PANEL_SHADOW: Color = Color::srgba(0.125, 0.094, 0.082, 0.85);
pub const TITLE_TEXT: Color = Color::srgb(1.0, 0.902, 0.753);
pub const STATUS_TEXT: Color = Color::srgb(0.90, 0.84, 0.76);

pub fn panel_bundle(width: f32, height: f32, offset_x: f32, offset_y: f32) -> impl Bundle {
    (
        Node {
            width: Val::Px(width),
            height: Val::Px(height),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(offset_x),
                top: Val::Px(offset_y),
                ..default()
            },
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(4.0)),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        BorderColor {
            top: PANEL_LIGHT_EDGE,
            left: PANEL_LIGHT_EDGE,
            right: PANEL_DARK_EDGE,
            bottom: PANEL_DARK_EDGE,
        },
        BoxShadow::new(
            PANEL_SHADOW,
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(3.0),
        ),
    )
}

pub fn absolute_text_bundle(
    value: impl Into<String>,
    font_size: f32,
    color: Color,
    left: Option<Val>,
    right: Option<Val>,
    top: Option<Val>,
    bottom: Option<Val>,
) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: default_font_size(font_size),
            ..default()
        },
        TextColor(color),
        Node {
            position_type: PositionType::Absolute,
            left: left.unwrap_or(Val::Auto),
            right: right.unwrap_or(Val::Auto),
            top: top.unwrap_or(Val::Auto),
            bottom: bottom.unwrap_or(Val::Auto),
            ..default()
        },
    )
}
