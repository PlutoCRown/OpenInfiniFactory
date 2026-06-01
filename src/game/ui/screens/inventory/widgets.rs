use bevy::prelude::*;

use crate::game::ui::components::{default_button_size, inset_border, label_text, styled_button};
use crate::game::ui::types::{AreaKind, InventoryItem, InventorySlot, SlotArea};

pub(super) fn spawn_slot(parent: &mut ChildSpawnerCommands, area: SlotArea, index: usize) {
    parent
        .spawn((
            styled_button(
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
                    ..default()
                },
                inset_border(),
                Color::srgb(0.255, 0.251, 0.251),
            ),
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.50),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(3.0),
            ),
            InventorySlot { area, index },
        ))
        .with_children(|slot| {
            slot.spawn((
                ImageNode::default(),
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
            slot.spawn((
                label_text("", 12.0, Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
            ));
        });
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
