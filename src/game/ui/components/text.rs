use bevy::prelude::*;

const DEFAULT_BUTTON_SCALE: f32 = 1.2;
const DEFAULT_TEXT_SCALE: f32 = 1.5;

pub fn default_font_size(font_size: f32) -> FontSize {
    FontSize::Px(font_size * DEFAULT_TEXT_SCALE)
}

pub fn default_button_size(size: f32) -> f32 {
    size * DEFAULT_BUTTON_SCALE
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
