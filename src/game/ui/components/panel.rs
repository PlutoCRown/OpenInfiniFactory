use bevy::prelude::*;

use super::super::types::{
    LocalizedText, PanelCloseButton, PanelFlowLayout, PanelPosition, PanelTitleBar, PanelWindow,
};
use super::button::{HoverButton, raised_border};
use super::icon::spawn_close_icon;
use super::text::default_font_size;
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
    /// 为 true 时以 Display::None 生成（常驻面板先藏着）
    pub start_hidden: bool,
}

impl PanelOptions {
    pub const fn new(width: f32, title_key: &'static str) -> Self {
        Self {
            width,
            title_key,
            show_close: false,
            title_size: 26.0,
            start_hidden: false,
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

    pub const fn start_hidden(mut self) -> Self {
        self.start_hidden = true;
        self
    }
}

pub fn spawn_panel(
    root: &mut ChildSpawnerCommands,
    options: PanelOptions,
    markers: impl Bundle,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    spawn_panel_with_title(
        root,
        options,
        markers,
        i18n.t(options.title_key),
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
    spawn_panel_with_title(
        root,
        options,
        markers,
        i18n.t(options.title_key),
        title_marker,
        content,
    );
}

/// 标题文案在 spawn 时写好（须已 bind_ui_scope）
pub fn spawn_panel_with_title(
    root: &mut ChildSpawnerCommands,
    options: PanelOptions,
    markers: impl Bundle,
    title: impl Into<String>,
    title_marker: impl Component,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let title = title.into();
    root.spawn((
        panel_window_bundle(
            Val::Px(options.width),
            Val::Percent(100.0),
            options.start_hidden,
            true,
        ),
        GlobalZIndex(0),
        markers,
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|bar| {
            bar.spawn((panel_title_label(title, options.title_size), title_marker));
            if options.show_close {
                bar.spawn(panel_close_button())
                    .with_children(spawn_close_icon);
            }
        });
        panel.spawn(panel_content()).with_children(content);
    });
}

pub fn panel_raised_border() -> BorderColor {
    BorderColor {
        top: PANEL_LIGHT_EDGE,
        left: PANEL_LIGHT_EDGE,
        right: PANEL_DARK_EDGE,
        bottom: PANEL_DARK_EDGE,
    }
}

pub fn panel_inset_border() -> BorderColor {
    BorderColor {
        top: PANEL_DARK_EDGE,
        left: PANEL_DARK_EDGE,
        right: PANEL_LIGHT_EDGE,
        bottom: PANEL_LIGHT_EDGE,
    }
}

pub const PANEL_INSET_BG: Color = Color::srgb(0.14, 0.137, 0.141);
pub const INVENTORY_TRAY_PADDING: f32 = 8.0;
pub const INVENTORY_SLOT_GAP: f32 = 6.0;
const INVENTORY_TRAY_BORDER: f32 = 3.0;

pub fn inventory_tray_row_bundle() -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(INVENTORY_SLOT_GAP),
            padding: UiRect::all(Val::Px(INVENTORY_TRAY_PADDING)),
            border: UiRect::all(Val::Px(INVENTORY_TRAY_BORDER)),
            ..default()
        },
        BackgroundColor(PANEL_INSET_BG),
        panel_inset_border(),
    )
}

pub fn inventory_tray_bundle() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(INVENTORY_SLOT_GAP),
            column_gap: Val::Px(INVENTORY_SLOT_GAP),
            padding: UiRect::all(Val::Px(INVENTORY_TRAY_PADDING)),
            border: UiRect::all(Val::Px(INVENTORY_TRAY_BORDER)),
            ..default()
        },
        BackgroundColor(PANEL_INSET_BG),
        panel_inset_border(),
    )
}

pub fn compact_raised_panel(style: Node) -> impl Bundle {
    (
        style,
        BackgroundColor(PANEL_BG),
        panel_raised_border(),
        BoxShadow::new(
            PANEL_SHADOW,
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(3.0),
        ),
    )
}

pub fn panel_bundle(width: f32) -> impl Bundle {
    panel_window_bundle(Val::Px(width), Val::Percent(100.0), false, true)
}

pub fn panel_bundle_auto(max_width_px: f32) -> impl Bundle {
    panel_window_bundle(Val::Auto, Val::Px(max_width_px), false, true)
}

pub fn panel_bundle_responsive(width_percent: f32, max_width_px: f32) -> impl Bundle {
    panel_window_bundle(
        Val::Percent(width_percent),
        Val::Px(max_width_px),
        false,
        true,
    )
}

/// 流式布局面板（相对定位），用于并排的多个面板
pub fn panel_bundle_responsive_flow(
    width_percent: f32,
    max_width_px: f32,
    start_hidden: bool,
) -> impl Bundle {
    (
        panel_window_bundle(
            Val::Percent(width_percent),
            Val::Px(max_width_px),
            start_hidden,
            false,
        ),
        PanelFlowLayout,
    )
}

fn panel_window_bundle(
    width: Val,
    max_width: Val,
    start_hidden: bool,
    absolute: bool,
) -> impl Bundle {
    (
        Node {
            width,
            height: Val::Auto,
            max_width,
            max_height: Val::Percent(100.0),
            position_type: if absolute {
                PositionType::Absolute
            } else {
                PositionType::Relative
            },
            left: Val::Auto,
            right: Val::Auto,
            top: Val::Auto,
            bottom: Val::Auto,
            margin: if absolute {
                UiRect::all(Val::Auto)
            } else {
                UiRect::all(Val::Px(0.0))
            },
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(4.0)),
            display: if start_hidden {
                Display::None
            } else {
                Display::Flex
            },
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            overflow: Overflow::clip(),
            ..default()
        },
        PanelWindow,
        PanelPosition::default(),
        if start_hidden {
            Visibility::Hidden
        } else {
            Visibility::Visible
        },
        BackgroundColor(PANEL_BG),
        panel_raised_border(),
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
        ZIndex(10),
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
        Pickable::IGNORE,
    )
}
