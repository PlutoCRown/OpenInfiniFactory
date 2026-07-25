use bevy::prelude::*;

use super::super::components::{
    INVENTORY_SLOT_GAP, INVENTORY_TRAY_PADDING, PanelOptions, compact_raised_panel,
    default_button_size, inventory_tray_row_bundle, spawn_panel_with_title, text,
};
use super::super::types::{
    BACKPACK_SLOTS, CarriedItemPreview, GameplayHudVisibility, HOTBAR_SLOTS, InGameHudStyle,
    ItemTooltip, ItemTooltipDescription, ItemTooltipName, PanelCloseButton, PanelVisibility,
    SlotArea,
};
use super::super::widgets::spawn_slot;
use crate::game::state::BuilderMode;
use crate::game::ui::access::i18n;
use crate::game::ui::features::inventory::InventoryTitleText;
use crate::shared::touch_profile::TouchProfile;

/// 背包一行格数；面板宽度按此精确排满
const BACKPACK_COLS: usize = 10;
/// 与 spawn_slot 一致
const SLOT_BASE: f32 = 54.0;
/// 与 panel_content / panel_window 一致
const PANEL_PAD: f32 = 8.0;
const PANEL_BORDER: f32 = 4.0;
const PANEL_BODY_BORDER: f32 = 3.0;

/// 使一行刚好放下 BACKPACK_COLS 个格子（含 gap、内外边距与边框）
fn inventory_panel_width() -> f32 {
    let slot = default_button_size(SLOT_BASE);
    let cols = BACKPACK_COLS as f32;
    let gaps = (BACKPACK_COLS.saturating_sub(1) as f32) * INVENTORY_SLOT_GAP;
    cols * slot
        + gaps
        + INVENTORY_TRAY_PADDING * 2.0
        + PANEL_PAD * 2.0
        + PANEL_BODY_BORDER * 2.0
        + PANEL_PAD * 2.0
        + PANEL_BORDER * 2.0
}

pub fn spawn_hotbar(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            bottom: Val::Px(0.0),
            width: Val::Percent(100.0),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        InGameHudStyle,
        GameplayHudVisibility,
    ))
    .with_children(|anchor| {
        // 高于视角层(0)、低于虚拟按键(10)，触屏才能点到快捷栏
        anchor
            .spawn((
                compact_raised_panel(Node {
                    border: UiRect::all(Val::Px(3.0)),
                    padding: UiRect::all(Val::Px(3.0)),
                    ..default()
                }),
                GlobalZIndex(5),
            ))
            .with_children(|outer| {
                outer
                    .spawn(inventory_tray_row_bundle())
                    .with_children(|bar| {
                        for index in 0..HOTBAR_SLOTS {
                            spawn_slot(bar, SlotArea::Hotbar, index);
                        }
                    });
            });
    });
}

pub fn spawn_inventory_panel(
    root: &mut ChildSpawnerCommands,
    builder_mode: BuilderMode,
    touch: TouchProfile,
) {
    let title = {
        let mode = i18n.t(match builder_mode {
            BuilderMode::Edit => "mode.edit",
            BuilderMode::Play => "mode.play",
        });
        i18n.fmt("inventory.title", &[("mode", mode.as_str())])
    };
    let mut options = PanelOptions::new(inventory_panel_width(), "inventory.title").start_hidden();
    // 触控无 Esc，标题栏需要关钮
    if touch.enabled {
        options = options.closable();
    }
    spawn_panel_with_title(
        root,
        options,
        PanelVisibility::Inventory,
        title,
        InventoryTitleText,
        PanelCloseButton,
        |panel| {
            // Panel 已有内凹，格子直接排，不再套一层凹陷框
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        row_gap: Val::Px(INVENTORY_SLOT_GAP),
                        column_gap: Val::Px(INVENTORY_SLOT_GAP),
                        padding: UiRect::all(Val::Px(INVENTORY_TRAY_PADDING)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|grid| {
                    for index in 0..BACKPACK_SLOTS {
                        spawn_slot(grid, SlotArea::Backpack, index);
                    }
                });
        },
    );
}

/// 手持物品预览：纯 Icon，居中跟在光标下
pub fn spawn_carried_label(root: &mut ChildSpawnerCommands) {
    let size = default_button_size(46.0);
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(size),
            height: Val::Px(size),
            display: Display::None,
            ..default()
        },
        ImageNode::default(),
        BackgroundColor(Color::NONE),
        ZIndex(10_000),
        GlobalZIndex(10_000),
        Pickable::IGNORE,
        CarriedItemPreview,
    ));
}

pub fn spawn_item_tooltip(root: &mut ChildSpawnerCommands) {
    // 约十余汉字宽，超出换行
    const MAX_WIDTH: f32 = 252.0;
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            max_width: Val::Px(MAX_WIDTH),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.72, 0.82, 0.88, 0.75)),
        BackgroundColor(Color::srgba(0.05, 0.06, 0.07, 0.92)),
        GlobalZIndex(30_000),
        Visibility::Hidden,
        Pickable::IGNORE,
        ItemTooltip,
    ))
    .with_children(|tooltip| {
        tooltip.spawn((
            text("", 14.0, Color::WHITE),
            ItemTooltipName,
            Pickable::IGNORE,
            Node {
                max_width: Val::Percent(100.0),
                ..default()
            },
        ));
        tooltip.spawn((
            text("", 12.0, Color::srgb(0.62, 0.62, 0.60)),
            ItemTooltipDescription,
            Pickable::IGNORE,
            Node {
                max_width: Val::Percent(100.0),
                ..default()
            },
        ));
    });
}
