use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::state::GameMode;
use crate::shared::i18n::I18n;

use crate::game::ui::components::{
    default_button_size, default_font_size, spawn_panel, PanelOptions,
};
use crate::game::ui::types::{
    CarriedItemPreview, GameplayHudVisibility, InGameHudStyle, InventoryTooltip, LocalizedText,
    InventoryRuntimeEntity, PanelText, PanelTextKind, PanelVisibility, SlotArea, UiPanelBinding,
    UiPanelKey, BACKPACK_SLOTS, HOTBAR_SLOTS,
};

mod actions;
mod carried_item;
mod render;
mod widgets;

pub use actions::inventory_slot_clicks;
pub use carried_item::update_carried_item_ui;
pub use render::update_inventory_slots;

use widgets::spawn_slot;

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
        InventoryRuntimeEntity,
    ))
    .with_children(|bar| {
        for index in 0..HOTBAR_SLOTS {
            spawn_slot(bar, SlotArea::Hotbar, index);
        }
    });
}

pub fn spawn_inventory_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(640.0, "inventory.title")
            .title_marker(PanelText(PanelTextKind::InventoryTitle)),
        (
            PanelVisibility::GameMode(GameMode::Inventory),
            UiPanelBinding(UiPanelKey::INVENTORY),
        ),
        |panel| {
            panel
                .spawn_empty()
                .queue_apply_scene(inventory_grid_scene())
                .with_children(|grid| {
                    for index in 0..BACKPACK_SLOTS {
                        spawn_slot(grid, SlotArea::Backpack, index);
                    }
                });
            panel
                .spawn(LocalizedText {
                    key: "inventory.help",
                })
                .queue_apply_scene(inventory_help_text_scene(i18n));
        },
    )
}

pub fn spawn_carried_label(root: &mut ChildSpawnerCommands) {
    root.spawn((CarriedItemPreview, InventoryRuntimeEntity))
        .queue_apply_scene(carried_item_preview_scene())
        .with_children(|icon| {
            icon.spawn_empty()
                .queue_apply_scene(carried_item_icon_scene());
            icon.spawn_empty()
                .queue_apply_scene(carried_item_label_scene());
        });
}

pub fn spawn_inventory_tooltip(root: &mut ChildSpawnerCommands) {
    root.spawn((InventoryTooltip, InventoryRuntimeEntity))
        .queue_apply_scene(inventory_tooltip_scene())
        .with_children(|tooltip| {
            tooltip
                .spawn_empty()
                .queue_apply_scene(inventory_tooltip_text_scene());
        });
}

fn carried_item_preview_scene() -> impl bevy_scene::Scene {
    bsn! {
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
        }
        BorderColor {
            top: Color::srgb(1.0, 1.0, 1.0),
            right: Color::srgb(1.0, 1.0, 1.0),
            bottom: Color::srgb(1.0, 1.0, 1.0),
            left: Color::srgb(1.0, 1.0, 1.0),
        }
        BackgroundColor(Color::srgba(0.18, 0.18, 0.19, 0.86))
        ZIndex(10_000)
        GlobalZIndex(10_000)
        Pickable::IGNORE
    }
}

fn inventory_help_text_scene(i18n: &I18n) -> impl bevy_scene::Scene {
    let value = i18n.text("inventory.help");

    bsn! {
        Text({value})
        TextFont {
            font_size: default_font_size(15.0)
        }
        TextColor(Color::srgb(0.78, 0.78, 0.76))
    }
}

fn carried_item_icon_scene() -> impl bevy_scene::Scene {
    bsn! {
        ImageNode::default()
        Pickable::IGNORE
        Node {
            width: Val::Px(default_button_size(64.0)),
            height: Val::Px(default_button_size(64.0)),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: {Val::Px(-default_button_size(32.0))},
                top: {Val::Px(-default_button_size(32.0))},
            },
        }
    }
}

fn carried_item_label_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        Pickable::IGNORE
        TextFont {
            font_size: {default_font_size(12.0)}
        }
        TextColor(Color::WHITE)
        TextLayout::justify(Justify::Center)
        Node {
            margin: UiRect::all(Val::Px(2.0)),
        }
    }
}

fn inventory_tooltip_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            display: Display::None,
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
        }
        BorderColor {
            top: Color::srgba(0.72, 0.82, 0.88, 0.75),
            right: Color::srgba(0.72, 0.82, 0.88, 0.75),
            bottom: Color::srgba(0.72, 0.82, 0.88, 0.75),
            left: Color::srgba(0.72, 0.82, 0.88, 0.75),
        }
        BackgroundColor(Color::srgba(0.05, 0.06, 0.07, 0.92))
        ZIndex(10_001)
        GlobalZIndex(10_001)
        Pickable::IGNORE
    }
}

fn inventory_tooltip_text_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        Pickable::IGNORE
        TextFont {
            font_size: {default_font_size(14.0)}
        }
        TextColor(Color::WHITE)
        TextLayout::justify(Justify::Center)
    }
}

fn inventory_grid_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(4.0),
            column_gap: Val::Px(4.0),
            width: Val::Percent(100.0),
            min_height: Val::Px(default_button_size(58.0)),
        }
        BackgroundColor(Color::NONE)
    }
}
