use bevy::prelude::*;

use crate::shared::i18n::I18n;

use super::types::LocalizedText;

pub const BUTTON_BG: Color = Color::srgb(0.56, 0.56, 0.56);
pub const BUTTON_HOVER_BG: Color = Color::srgb(0.68, 0.68, 0.68);
pub const BUTTON_PRESSED_BG: Color = Color::srgb(0.40, 0.40, 0.40);
pub const BUTTON_BORDER: Color = Color::srgb(0.12, 0.10, 0.09);
pub const BUTTON_LIGHT_EDGE: Color = Color::srgb(0.89, 0.89, 0.89);
pub const BUTTON_DARK_EDGE: Color = Color::srgb(0.02, 0.02, 0.02);
const DEFAULT_BUTTON_SCALE: f32 = 1.2;
const DEFAULT_TEXT_SCALE: f32 = 1.5;

pub fn default_font_size(font_size: f32) -> f32 {
    font_size * DEFAULT_TEXT_SCALE
}

pub fn default_button_size(size: f32) -> f32 {
    size * DEFAULT_BUTTON_SCALE
}

pub fn transparent_node(style: Node) -> impl Bundle {
    (style, BackgroundColor(Color::NONE))
}

pub fn text(value: impl Into<String>, font_size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: default_font_size(font_size),
            ..default()
        },
        TextColor(color),
    )
}

pub fn localized_text(
    i18n: &I18n,
    key: &'static str,
    font_size: f32,
    color: Color,
) -> (impl Bundle, LocalizedText) {
    (
        text(i18n.text(key), font_size, color),
        LocalizedText { key },
    )
}

pub fn menu_button(height: f32) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Percent(100.0),
            min_width: Val::Px(default_button_size(92.0)),
            height: Val::Px(default_button_size(height)),
            border: UiRect {
                left: Val::Px(3.0),
                right: Val::Px(3.0),
                top: Val::Px(4.0),
                bottom: Val::Px(5.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        raised_border(),
        BackgroundColor(BUTTON_BG),
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.62),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(4.0),
        ),
    )
}

pub fn raised_border() -> BorderColor {
    BorderColor {
        top: BUTTON_LIGHT_EDGE,
        left: BUTTON_LIGHT_EDGE,
        right: BUTTON_DARK_EDGE,
        bottom: BUTTON_DARK_EDGE,
    }
}

pub fn pressed_border() -> BorderColor {
    BorderColor {
        top: BUTTON_DARK_EDGE,
        left: BUTTON_DARK_EDGE,
        right: BUTTON_LIGHT_EDGE,
        bottom: BUTTON_LIGHT_EDGE,
    }
}

pub fn hover_border() -> BorderColor {
    BorderColor {
        top: Color::srgb(1.0, 1.0, 1.0),
        left: Color::srgb(0.92, 0.92, 0.92),
        right: Color::srgb(0.08, 0.08, 0.08),
        bottom: Color::srgb(0.04, 0.04, 0.04),
    }
}

pub fn inset_border() -> BorderColor {
    BorderColor {
        top: BUTTON_DARK_EDGE,
        left: BUTTON_DARK_EDGE,
        right: BUTTON_LIGHT_EDGE,
        bottom: BUTTON_LIGHT_EDGE,
    }
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
