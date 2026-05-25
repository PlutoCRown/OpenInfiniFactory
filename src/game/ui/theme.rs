use bevy::prelude::*;

pub const PANEL_BG: Color = Color::srgba(0.08, 0.09, 0.10, 0.96);
pub const BUTTON_BG: Color = Color::srgba(0.22, 0.24, 0.26, 0.96);
pub const BUTTON_BORDER: Color = Color::srgb(0.38, 0.39, 0.40);
pub const STATUS_TEXT: Color = Color::srgb(0.88, 0.96, 1.0);

pub fn panel_bundle(width: f32, height: f32, offset_x: f32, offset_y: f32) -> NodeBundle {
    NodeBundle {
        style: Style {
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
            padding: UiRect::all(Val::Px(20.0)),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(14.0),
            ..default()
        },
        background_color: PANEL_BG.into(),
        ..default()
    }
}

pub fn row_bundle(height: f32) -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(height),
            display: Display::Flex,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },
        background_color: Color::NONE.into(),
        ..default()
    }
}

pub fn text_section(value: impl Into<String>, font_size: f32, color: Color) -> TextBundle {
    TextBundle::from_section(
        value,
        TextStyle {
            font_size,
            color,
            ..default()
        },
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
) -> TextBundle {
    TextBundle {
        text: Text::from_section(
            value,
            TextStyle {
                font_size,
                color,
                ..default()
            },
        ),
        style: Style {
            position_type: PositionType::Absolute,
            left: left.unwrap_or(Val::Auto),
            right: right.unwrap_or(Val::Auto),
            top: top.unwrap_or(Val::Auto),
            bottom: bottom.unwrap_or(Val::Auto),
            ..default()
        },
        ..default()
    }
}
