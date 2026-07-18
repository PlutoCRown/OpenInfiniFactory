use bevy::prelude::*;

use super::super::types::LocalizedText;
use crate::game::ui::access::i18n;

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

/// 生成本地化文本；须在 `UiAccessScope` / 已 `bind_ui_scope` 下调用，spawn 时直接填好文案
pub fn localized_text(
    key: &'static str,
    font_size: f32,
    color: Color,
) -> (impl Bundle, LocalizedText) {
    (
        text(i18n.t(key), font_size, color),
        LocalizedText { key },
    )
}
