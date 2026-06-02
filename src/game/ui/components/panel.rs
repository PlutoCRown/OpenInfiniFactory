use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use super::super::types::{
    BlockPanelText, LocalizedText, PanelCloseButton, PanelPosition, PanelText, PanelTitleBar,
    PanelWindow,
};
use super::button::{raised_border, HoverButton, BUTTON_BG};
use super::text::default_font_size;
use crate::shared::i18n::I18n;

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
    pub title_marker: Option<PanelText>,
    pub block_title_marker: Option<BlockPanelText>,
}

impl PanelOptions {
    pub const fn new(width: f32, title_key: &'static str) -> Self {
        Self {
            width,
            title_key,
            show_close: false,
            title_size: 26.0,
            title_marker: None,
            block_title_marker: None,
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

    pub const fn title_marker(mut self, marker: PanelText) -> Self {
        self.title_marker = Some(marker);
        self
    }

    pub const fn block_title_marker(mut self, marker: BlockPanelText) -> Self {
        self.block_title_marker = Some(marker);
        self
    }
}

pub fn spawn_panel(
    root: &mut ChildSpawnerCommands,
    i18n: &I18n,
    options: PanelOptions,
    markers: impl Bundle,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    root.spawn((
        GlobalZIndex(0),
        PanelWindow,
        PanelPosition::default(),
        Visibility::Hidden,
        markers,
    ))
    .queue_apply_scene(panel_window_scene(options.width))
    .with_children(|panel| {
        panel
            .spawn((Button, PanelTitleBar, Visibility::Visible))
            .queue_apply_scene(panel_title_bar_scene())
            .with_children(|title| {
                let mut title_text = title.spawn(Visibility::Visible);
                title_text.queue_apply_scene(panel_title_label_scene(
                    i18n.text(options.title_key),
                    options.title_size,
                ));
                if let Some(marker) = options.title_marker {
                    title_text.insert(marker);
                } else if let Some(marker) = options.block_title_marker {
                    title_text.insert(marker);
                } else {
                    title_text.insert(LocalizedText {
                        key: options.title_key,
                    });
                }
                if options.show_close {
                    title
                        .spawn((Button, HoverButton, PanelCloseButton, Visibility::Visible))
                        .queue_apply_scene(panel_title_button_scene())
                        .queue_spawn_related_scenes::<Children>(panel_close_label_scene());
                }
            });
        panel
            .spawn(Visibility::Visible)
            .queue_apply_scene(panel_content_scene())
            .with_children(content);
    })
    .id()
}

pub fn panel_window_scene(width: f32) -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Px(width),
            height: Val::Auto,
            max_width: Val::Percent(100.0),
            max_height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            bottom: Val::Px(10.0),
            margin: UiRect::all(Val::Auto),
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(4.0)),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            overflow: Overflow::scroll_y(),
        }
        BackgroundColor(PANEL_BG)
        BorderColor {
            top: PANEL_LIGHT_EDGE,
            left: PANEL_LIGHT_EDGE,
            right: PANEL_DARK_EDGE,
            bottom: PANEL_DARK_EDGE,
        }
        BoxShadow::new(
            PANEL_SHADOW,
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(3.0),
        )
        Pickable::IGNORE
    }
}

pub fn panel_title_bar_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(38.0),
            padding: UiRect::horizontal(Val::Px(10.0)),
            border: UiRect {
                bottom: Val::Px(2.0),
            },
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(10.0),
            flex_shrink: 0.0,
        }
        BackgroundColor(Color::NONE)
        Visibility::Visible
        BorderColor {
            bottom: PANEL_DARK_EDGE,
        }
        Pickable {
            should_block_lower: true,
            is_hoverable: true,
        }
    }
}

pub fn panel_title_label_scene(value: String, font_size: f32) -> impl bevy_scene::Scene {
    bsn! {
        Text({value})
        Pickable::IGNORE
        TextFont {
            font_size: {default_font_size(font_size * TITLE_FONT_SCALE)}
        }
        TextColor(TITLE_TEXT)
        Node {
            flex_grow: 1.0,
        }
    }
}

pub fn panel_title_button_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Px(28.0),
            height: Val::Px(28.0),
            border: UiRect::all(Val::Px(2.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_shrink: 0.0,
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(BUTTON_BG)
        Visibility::Visible
    }
}

pub fn panel_close_label_scene() -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text("x")
            TextFont {
                font_size: {default_font_size(12.0)}
            }
            TextColor(Color::WHITE)
            Pickable::IGNORE
        )
    }
}

pub fn panel_content_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(8.0)),
        }
        BackgroundColor(Color::NONE)
        Visibility::Visible
    }
}
