use bevy::prelude::*;

use super::super::types::LocalizedText;

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

pub fn localized_text(
    key: &'static str,
    font_size: f32,
    color: Color,
) -> (impl Bundle, LocalizedText) {
    // 不在 spawn 时查 i18n：挂载可能发生在任意系统/线程，留给 update_localized_ui 填文案
    (
        text("", font_size, color),
        LocalizedText { key },
    )
}
