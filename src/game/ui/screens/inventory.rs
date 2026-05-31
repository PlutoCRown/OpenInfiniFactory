use bevy::prelude::*;

use crate::game::state::GameMode;
use crate::shared::i18n::I18n;

use super::super::components::{
    default_button_size, default_font_size, localized_text, spawn_panel, text, transparent_node,
    PanelOptions,
};
use super::super::types::{
    CarriedItemPreview, GameplayHudVisibility, InGameHudStyle, InventoryTooltip, PanelText,
    PanelTextKind, PanelVisibility, SlotArea, BACKPACK_SLOTS, HOTBAR_SLOTS,
};
use super::super::widgets::spawn_slot;

pub fn spawn_hotbar(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Px(default_button_size(540.0)),
            height: Val::Px(default_button_size(58.0)),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            bottom: Val::Px(22.0),
            margin: UiRect {
                left: Val::Px(-default_button_size(270.0)),
                ..default()
            },
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(4.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.38)),
        InGameHudStyle,
        GameplayHudVisibility,
    ))
    .with_children(|bar| {
        for index in 0..HOTBAR_SLOTS {
            spawn_slot(bar, SlotArea::Hotbar, index);
        }
    });
}

pub fn spawn_inventory_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(640.0, "inventory.title")
            .title_marker(PanelText(PanelTextKind::InventoryTitle)),
        PanelVisibility::GameMode(GameMode::Inventory),
        |panel| {
            panel.spawn(inventory_grid_bundle()).with_children(|grid| {
                for index in 0..BACKPACK_SLOTS {
                    spawn_slot(grid, SlotArea::Backpack, index);
                }
            });
            panel.spawn(localized_text(
                i18n,
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
            TextLayout::new_with_justify(Justify::Center),
            Node {
                margin: UiRect::all(Val::Px(2.0)),
                ..default()
            },
        ));
    });
}

pub fn spawn_inventory_tooltip(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            display: Display::None,
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
            Pickable::IGNORE,
            TextLayout::new_with_justify(Justify::Center),
        ));
    });
}

fn inventory_grid_bundle() -> impl Bundle {
    transparent_node(Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        row_gap: Val::Px(4.0),
        column_gap: Val::Px(4.0),
        width: Val::Percent(100.0),
        min_height: Val::Px(default_button_size(58.0)),
        ..default()
    })
}
