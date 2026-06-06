use bevy::prelude::*;

use super::super::types::{
    LocalizedText, PanelCloseButton, PanelPosition, PanelTitleBar, PanelWindow,
};
use super::button::{raised_border, HoverButton};
use super::text::{default_font_size, text};
use crate::game::ui::access::i18n;

pub const PANEL_BG: Color = Color::srgb(0.192, 0.188, 0.192);
pub const PANEL_LIGHT_EDGE: Color = Color::srgb(0.40, 0.38, 0.36);
pub const PANEL_DARK_EDGE: Color = Color::srgb(0.08, 0.06, 0.05);
pub const PANEL_SHADOW: Color = Color::srgba(0.125, 0.094, 0.082, 0.85);
pub const TITLE_TEXT: Color = Color::srgb(1.0, 0.902, 0.753);
pub const STATUS_TEXT: Color = Color::srgb(0.90, 0.84, 0.76);
const TITLE_FONT_SCALE: f32 = 0.8;

#[derive(Clone, Copy)]
pub struct PanelOptions {
    pub width: f32,
    pub title_key: &'static str,
    pub show_close: bool,
    pub title_size: f32,
    pub dynamic_title: bool,
}

impl PanelOptions {
    pub const fn new(width: f32, title_key: &'static str) -> Self {
        Self {
            width,
            title_key,
            show_close: false,
            title_size: 26.0,
            dynamic_title: false,
        }
    }

    pub const fn closable(mut self) -> Self {
        self.show_close = true;
        self
    }

    pub const fn title_size(mut self, title_size: f32) -> Self {
        self.title_size = title_size;
        self
    }

    pub const fn dynamic_title(mut self) -> Self {
        self.dynamic_title = true;
        self
    }
}

pub fn spawn_panel(
    root: &mut ChildSpawnerCommands,
    options: PanelOptions,
    markers: impl Bundle,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    spawn_panel_with_title_marker(
        root,
        options,
        markers,
        LocalizedText {
            key: options.title_key,
        },
        content,
    );
}

pub fn spawn_panel_with_title_marker(
    root: &mut ChildSpawnerCommands,
    options: PanelOptions,
    markers: impl Bundle,
    title_marker: impl Component,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    root.spawn((panel_bundle(options.width), GlobalZIndex(0), markers))
        .with_children(|panel| {
            panel.spawn(panel_title_bar()).with_children(|title| {
                let mut title_text = title.spawn(panel_title_label(
                    i18n.t(options.title_key),
                    options.title_size,
                ));
                if options.dynamic_title {
                    title_text.insert(crate::game::block_editing::BlockPanelTitle);
                } else {
                    title_text.insert(title_marker);
                }
                if options.show_close {
                    title.spawn(panel_close_button()).with_children(|button| {
                        button.spawn(text("x", 12.0, Color::WHITE));
                    });
                }
            });
            panel.spawn(panel_content()).with_children(content);
        });
}

pub fn panel_bundle(width: f32) -> impl Bundle {
    (
        Node {
            width: Val::Px(width),
            height: Val::Auto,
            max_width: Val::Percent(100.0),
            max_height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Auto,
            right: Val::Auto,
            top: Val::Auto,
            bottom: Val::Auto,
            margin: UiRect::all(Val::Auto),
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(4.0)),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        PanelWindow,
        PanelPosition::default(),
        Visibility::Hidden,
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
        Pickable {
            should_block_lower: true,
            is_hoverable: false,
        },
    )
}

pub fn panel_title_bar() -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(38.0),
            padding: UiRect::horizontal(Val::Px(10.0)),
            border: UiRect {
                bottom: Val::Px(2.0),
                ..default()
            },
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(10.0),
            flex_shrink: 0.0,
            ..default()
        },
        PanelTitleBar,
        BackgroundColor(Color::NONE),
        BorderColor {
            bottom: PANEL_DARK_EDGE,
            ..Default::default()
        },
        Pickable {
            should_block_lower: true,
            is_hoverable: true,
        },
    )
}

pub fn panel_title_label(value: impl Into<String>, font_size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: default_font_size(font_size * TITLE_FONT_SCALE),
            ..default()
        },
        TextColor(TITLE_TEXT),
        Node {
            flex_grow: 1.0,
            ..default()
        },
    )
}

pub fn panel_close_button() -> impl Bundle {
    (panel_title_button(), PanelCloseButton)
}

pub fn panel_title_button() -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Px(28.0),
            height: Val::Px(28.0),
            border: UiRect::all(Val::Px(2.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_shrink: 0.0,
            ..default()
        },
        HoverButton,
        raised_border(),
        BackgroundColor(super::button::BUTTON_BG),
    )
}

pub fn panel_content() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
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
