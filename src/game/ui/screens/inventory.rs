use bevy::prelude::*;

use super::super::components::{
    compact_raised_panel, default_button_size, default_font_size, inventory_tray_bundle,
    inventory_tray_row_bundle, localized_text, spawn_panel_with_title_marker, text, PanelOptions,
};
use super::super::types::{
    CarriedItemPreview, GameplayHudVisibility, InGameHudStyle, InventoryTooltip,
    InventoryTooltipDescription, InventoryTooltipName, PanelVisibility, SlotArea, BACKPACK_SLOTS,
    HOTBAR_SLOTS,
};
use super::super::widgets::spawn_slot;
use crate::game::ui::features::inventory::InventoryTitleText;

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

pub fn spawn_inventory_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel_with_title_marker(
        root,
        PanelOptions::new(640.0, "inventory.title"),
        PanelVisibility::Inventory,
        InventoryTitleText,
        |panel| {
            panel.spawn(inventory_tray_bundle()).with_children(|grid| {
                for index in 0..BACKPACK_SLOTS {
                    spawn_slot(grid, SlotArea::Backpack, index);
                }
            });
            panel.spawn(localized_text(
                "inventory.help",
                15.0,
                Color::srgb(0.78, 0.78, 0.76),
            ));
        },
    );
}

pub fn spawn_carried_label(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(default_button_size(46.0)),
            height: Val::Px(default_button_size(46.0)),
            display: Display::None,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor::all(Color::srgb(1.0, 1.0, 1.0)),
        BackgroundColor(Color::srgba(0.18, 0.18, 0.19, 0.86)),
        ZIndex(10_000),
        GlobalZIndex(10_000),
        Pickable::IGNORE,
        CarriedItemPreview,
    ))
    .with_children(|icon| {
        icon.spawn((
            ImageNode::default(),
            Pickable::IGNORE,
            Node {
                width: Val::Px(default_button_size(64.0)),
                height: Val::Px(default_button_size(64.0)),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-default_button_size(32.0)),
                    top: Val::Px(-default_button_size(32.0)),
                    ..default()
                },
                ..default()
            },
        ));
        icon.spawn((
            Text::new(""),
            Pickable::IGNORE,
            TextFont {
                font_size: default_font_size(12.0),
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Center),
            Node {
                margin: UiRect::all(Val::Px(2.0)),
                ..default()
            },
        ));
    });
}

pub fn spawn_inventory_tooltip(root: &mut ChildSpawnerCommands) {
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
        ZIndex(10_001),
        GlobalZIndex(10_001),
        Pickable::IGNORE,
        InventoryTooltip,
    ))
    .with_children(|tooltip| {
        tooltip.spawn((
            text("", 14.0, Color::WHITE),
            InventoryTooltipName,
            Pickable::IGNORE,
            Node {
                max_width: Val::Percent(100.0),
                ..default()
            },
        ));
        tooltip.spawn((
            text("", 12.0, Color::srgb(0.62, 0.62, 0.60)),
            InventoryTooltipDescription,
            Pickable::IGNORE,
            Node {
                max_width: Val::Percent(100.0),
                ..default()
            },
        ));
    });
}
