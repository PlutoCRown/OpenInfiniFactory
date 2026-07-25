use bevy::prelude::*;

use super::super::types::{
    LocalizedText, PanelCloseButton, PanelFlowLayout, PanelPosition, PanelTitleBar, PanelTitleText,
    PanelWindow,
};
use super::button::{BUTTON_BG, HoverButton, button_border, raised_border};
use super::icon::{UiIconAssets, spawn_ui_icon};
use super::text::default_font_size;
use crate::game::ui::access::{i18n, with_ui_world};
use crate::game::ui::systems::UiFont;

pub const PANEL_BG: Color = Color::srgb(0.192, 0.188, 0.192);
pub const PANEL_LIGHT_EDGE: Color = Color::srgb(0.40, 0.38, 0.36);
pub const PANEL_DARK_EDGE: Color = Color::srgb(0.08, 0.06, 0.05);
pub const PANEL_SHADOW: Color = Color::srgba(0.125, 0.094, 0.082, 0.85);
pub const TITLE_TEXT: Color = Color::srgb(1.0, 0.902, 0.753);
pub const STATUS_TEXT: Color = Color::srgb(0.90, 0.84, 0.76);
const TITLE_FONT_SIZE: f32 = 14.0;
const TITLE_BAR_PAD: f32 = 2.0;
/// 关闭钮边长（可视与热区一致）
const TITLE_CLOSE_SIZE: f32 = 36.0;
const TITLE_CLOSE_ICON: f32 = 16.0;

#[derive(Clone, Copy)]
pub struct PanelOptions {
    pub width: f32,
    /// 固定高度；`None` 时按内容自适应
    pub height: Option<f32>,
    pub title_key: &'static str,
    pub show_close: bool,
    /// 为 true 时以 Display::None 生成（常驻面板先藏着）
    pub start_hidden: bool,
    /// 父级 flex 居中：Relative + PanelFlowLayout，拖动后再切 Absolute
    pub flow: bool,
}

impl PanelOptions {
    pub const fn new(width: f32, title_key: &'static str) -> Self {
        Self {
            width,
            height: None,
            title_key,
            show_close: false,
            start_hidden: false,
            flow: false,
        }
    }

    pub const fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub const fn closable(mut self) -> Self {
        self.show_close = true;
        self
    }

    pub const fn start_hidden(mut self) -> Self {
        self.start_hidden = true;
        self
    }

    pub const fn flow(mut self) -> Self {
        self.flow = true;
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
        PanelCloseButton,
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
        PanelCloseButton,
        content,
    );
}

/// 标题文案在 spawn 时写好（须已 bind_ui_scope）；`close_extras` 在 show_close 时挂到关闭钮
pub fn spawn_panel_with_title(
    root: &mut ChildSpawnerCommands,
    options: PanelOptions,
    markers: impl Bundle,
    title: impl Into<String>,
    title_marker: impl Component,
    close_extras: impl Bundle,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let title = title.into();
    let close_icon = options
        .show_close
        .then(|| with_ui_world(|world| world.resource::<UiIconAssets>().close.clone()));
    let mut entity = root.spawn((
        panel_window_bundle(
            Val::Px(options.width),
            options.height.map(Val::Px).unwrap_or(Val::Auto),
            Val::Percent(100.0),
            options.start_hidden,
            !options.flow,
        ),
        GlobalZIndex(0),
        markers,
    ));
    if options.flow {
        entity.insert(PanelFlowLayout);
    }
    entity.with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|bar| {
            bar.spawn((panel_title_label(title), title_marker));
            match close_icon {
                Some(close_icon) => spawn_panel_close(bar, close_icon, close_extras),
                None => {
                    let _ = close_extras;
                }
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
/// 面板主体内凹边框厚度
const PANEL_BODY_BORDER: f32 = 3.0;

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

pub fn panel_bundle_auto(max_width_px: f32) -> impl Bundle {
    panel_window_bundle(Val::Auto, Val::Auto, Val::Px(max_width_px), false, true)
}

/// 固定宽度、高度随内容（文本输入等需要稳定宽栏时用）
pub fn panel_bundle(width_px: f32) -> impl Bundle {
    panel_window_bundle(
        Val::Px(width_px),
        Val::Auto,
        Val::Percent(100.0),
        false,
        true,
    )
}

fn panel_window_bundle(
    width: Val,
    height: Val,
    max_width: Val,
    start_hidden: bool,
    absolute: bool,
) -> impl Bundle {
    (
        Node {
            width,
            height,
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
            row_gap: Val::Px(8.0),
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
            // 四周等距，避免关闭按钮顶距与右边距不一致
            padding: UiRect::all(Val::Px(TITLE_BAR_PAD)),
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(8.0),
            flex_shrink: 0.0,
            ..default()
        },
        PanelTitleBar,
        ZIndex(10),
        BackgroundColor(Color::NONE),
        Pickable {
            should_block_lower: true,
            is_hoverable: true,
        },
    )
}

/// 面板标题文案（字号/加粗字重统一在此；MiSansVF 用 SEMIBOLD）
pub fn panel_title_label(value: impl Into<String>) -> impl Bundle {
    let font = with_ui_world(|world| world.resource::<UiFont>().0.clone());
    (
        Text::new(value),
        TextFont {
            font: font.into(),
            font_size: default_font_size(TITLE_FONT_SIZE),
            weight: FontWeight::SEMIBOLD,
            ..default()
        },
        TextColor(TITLE_TEXT),
        PanelTitleText,
        Node {
            flex_grow: 1.0,
            ..default()
        },
    )
}

/// 面板关闭钮
pub fn spawn_panel_close(
    parent: &mut ChildSpawnerCommands,
    image: Handle<Image>,
    extras: impl Bundle,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(TITLE_CLOSE_SIZE),
                height: Val::Px(TITLE_CLOSE_SIZE),
                border: button_border(),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_shrink: 0.0,
                ..default()
            },
            HoverButton,
            raised_border(),
            BackgroundColor(BUTTON_BG),
            extras,
        ))
        .with_children(|btn| {
            spawn_ui_icon(btn, image, TITLE_CLOSE_ICON);
        });
}

pub fn panel_content() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(PANEL_BODY_BORDER)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(PANEL_INSET_BG),
        panel_inset_border(),
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
