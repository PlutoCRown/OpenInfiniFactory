use bevy::prelude::*;

use super::super::components::{
    PanelOptions, compact_raised_panel, default_button_size, default_font_size,
    inventory_tray_bundle, inventory_tray_row_bundle, localized_text, spawn_panel_with_title, text,
};
use super::super::types::{
    BACKPACK_SLOTS, CarriedItemPreview, GameplayHudVisibility, HOTBAR_SLOTS, InGameHudStyle,
    InventoryTooltip, InventoryTooltipDescription, InventoryTooltipName, PanelVisibility, SlotArea,
};
use super::super::widgets::spawn_slot;
use crate::game::state::BuilderMode;
use crate::game::ui::access::i18n;
use crate::game::ui::features::inventory::InventoryTitleText;
use crate::shared::touch_profile::TouchProfile;

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
    let mut options = PanelOptions::new(640.0, "inventory.title").start_hidden();
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
        const ICON_INSET: f32 = 4.0;
        icon.spawn((
            ImageNode::default(),
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ICON_INSET),
                right: Val::Px(ICON_INSET),
                top: Val::Px(ICON_INSET),
                bottom: Val::Px(ICON_INSET),
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
        GlobalZIndex(30_000),
        Visibility::Hidden,
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
