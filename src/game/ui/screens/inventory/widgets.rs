use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::ui::components::{
    default_button_size, default_font_size, inset_border, HoverButton,
};
use crate::game::ui::types::{AreaKind, InventoryItem, InventorySlot, SlotArea};

use super::inventory_slot_clicks;

pub(super) fn spawn_slot(parent: &mut ChildSpawnerCommands, area: SlotArea, index: usize) {
    parent
        .spawn((Button, HoverButton, InventorySlot { area, index }))
        .observe(inventory_slot_clicks)
        .queue_apply_scene(inventory_slot_visual_scene())
        .with_children(|slot| {
            slot.spawn_empty()
                .queue_apply_scene(inventory_slot_icon_scene());
            slot.spawn_empty()
                .queue_apply_scene(inventory_slot_label_scene());
        });
}

fn inventory_slot_visual_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Px(default_button_size(54.0)),
            height: Val::Px(default_button_size(54.0)),
            border: UiRect {
                left: Val::Px(4.0),
                right: Val::Px(4.0),
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BorderColor {
            top: {inset_border().top},
            right: {inset_border().right},
            bottom: {inset_border().bottom},
            left: {inset_border().left},
        }
        BackgroundColor(Color::srgb(0.255, 0.251, 0.251))
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.50),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(3.0),
        )
    }
}

fn inventory_slot_icon_scene() -> impl bevy_scene::Scene {
    bsn! {
        ImageNode::default()
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

fn inventory_slot_label_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
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

pub(crate) fn slot_color(item: InventoryItem) -> Color {
    match item {
        InventoryItem::Area(AreaKind::Selection) => Color::srgb(0.22, 0.66, 0.62),
        InventoryItem::Block(kind) => kind.slot_color(),
    }
}

pub(crate) fn short_item_name(item: InventoryItem) -> &'static str {
    match item {
        InventoryItem::Area(AreaKind::Selection) => "short.area.selection",
        InventoryItem::Block(kind) => kind.short_name_key(),
    }
}
