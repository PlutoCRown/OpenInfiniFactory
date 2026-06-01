mod button;
mod layout;
mod panel;
mod scroll;
mod slider;
mod text;

use bevy::prelude::*;

pub use button::{
    button_hovered, button_pressed, button_released, button_unhovered, full_width_button,
    hover_border, inset_border, menu_button, pressed_border, raised_border, styled_button,
    BUTTON_BG, BUTTON_HOVER_BG,
};
pub use layout::{flex_row, root_node, transparent_node};
pub use panel::{
    absolute_text_bundle, panel_bundle, panel_content, panel_title_bar, panel_title_button,
    panel_title_label, spawn_panel, PanelOptions, STATUS_TEXT,
};
pub use scroll::{scroll_container, scroll_content, update_scroll_containers};
pub use slider::{slider_bundle, slider_fill, slider_knob};
pub use text::{default_button_size, default_font_size, localized_text, text};

pub fn label_text(value: impl Into<String>, font_size: f32, color: Color) -> impl Bundle {
    text(value, font_size, color)
}
