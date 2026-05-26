use bevy::prelude::*;

use crate::shared::i18n::I18n;

use super::types::LocalizedText;

const BUTTON_BG: Color = Color::srgba(0.22, 0.24, 0.26, 0.96);
const BUTTON_BORDER: Color = Color::srgb(0.38, 0.39, 0.40);
const DEFAULT_BUTTON_SCALE: f32 = 1.2;
const DEFAULT_TEXT_SCALE: f32 = 1.5;

pub fn default_font_size(font_size: f32) -> f32 {
    font_size * DEFAULT_TEXT_SCALE
}

pub fn default_button_size(size: f32) -> f32 {
    size * DEFAULT_BUTTON_SCALE
}

pub fn transparent_node(style: Style) -> NodeBundle {
    NodeBundle {
        style,
        background_color: Color::NONE.into(),
        ..default()
    }
}

pub fn text(value: impl Into<String>, font_size: f32, color: Color) -> TextBundle {
    TextBundle::from_section(
        value,
        TextStyle {
            font_size: default_font_size(font_size),
            color,
            ..default()
        },
    )
}

pub fn localized_text(
    i18n: &I18n,
    key: &'static str,
    font_size: f32,
    color: Color,
) -> (TextBundle, LocalizedText) {
    (
        text(i18n.text(key), font_size, color),
        LocalizedText { key },
    )
}

pub fn menu_button(height: f32) -> ButtonBundle {
    ButtonBundle {
        style: Style {
            width: Val::Percent(100.0),
            min_width: Val::Px(default_button_size(92.0)),
            height: Val::Px(default_button_size(height)),
            border: UiRect::all(Val::Px(1.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        border_color: BUTTON_BORDER.into(),
        background_color: BUTTON_BG.into(),
        ..default()
    }
}

pub fn root_node() -> NodeBundle {
    transparent_node(Style {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    })
}

pub fn flex_row(height: f32, column_gap: f32) -> NodeBundle {
    transparent_node(Style {
        width: Val::Percent(100.0),
        height: Val::Px(default_button_size(height)),
        display: Display::Flex,
        align_items: AlignItems::Center,
        column_gap: Val::Px(column_gap),
        ..default()
    })
}
