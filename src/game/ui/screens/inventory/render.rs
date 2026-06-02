use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::state::PlacementState;
use crate::game::ui::components::{hover_border, inset_border};
use crate::game::ui::types::{
    CarriedItemPreview, GameplayUiChanged, InventoryChanged, InventoryItems, InventorySlot,
    InventoryTooltip, LanguageChanged, SlotArea, UiHoverState, UiPanelContextChanged, UiPanelKey,
    UiPanelOpened,
};
use crate::game::world::rendering::BlockIconAssets;
use crate::shared::i18n::I18n;

use super::widgets::{short_item_name, slot_color};

#[derive(Clone, Copy)]
struct SlotRenderData {
    item: Option<crate::game::ui::types::InventoryItem>,
    has_icon: bool,
    hovered: bool,
    selected_hotbar: bool,
}

pub fn update_inventory_slots(
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    i18n: Res<I18n>,
    hover: Res<UiHoverState>,
    block_icons: Option<Res<BlockIconAssets>>,
    mut slot_query: Query<
        (
            Entity,
            &InventorySlot,
            &Children,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<Button>, Without<InventoryTooltip>),
    >,
    added_slots: Query<(), Added<InventorySlot>>,
    mut opened: MessageReader<UiPanelOpened>,
    mut context_changed: MessageReader<UiPanelContextChanged>,
    mut gameplay_ui_changed: MessageReader<GameplayUiChanged>,
    mut inventory_changed: MessageReader<InventoryChanged>,
    mut language_changed: MessageReader<LanguageChanged>,
    mut texts: ParamSet<(
        Query<&mut Text, (Without<CarriedItemPreview>, Without<InventoryTooltip>)>,
        Query<&mut Text, (With<InventoryTooltip>, Without<CarriedItemPreview>)>,
    )>,
    mut icons: Query<&mut ImageNode>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut tooltip: Query<(&mut Node, &Children), (With<InventoryTooltip>, Without<Button>)>,
) {
    let refresh_slots = gameplay_ui_changed.read().next().is_some()
        || inventory_changed.read().next().is_some()
        || language_changed.read().next().is_some()
        || hover.is_changed()
        || block_icons.as_ref().is_some_and(|icons| icons.is_changed())
        || !added_slots.is_empty()
        || opened
            .read()
            .any(|message| message.key == UiPanelKey::INVENTORY)
        || context_changed
            .read()
            .any(|message| message.key == UiPanelKey::INVENTORY);
    let mut hovered_item = None;
    for (entity, slot, children, mut node, mut background, mut border) in &mut slot_query {
        let item = match slot.area {
            SlotArea::Hotbar => inventory.hotbar[slot.index],
            SlotArea::Backpack => inventory.backpack[slot.index],
        };
        let icon_handle = item
            .and_then(|item| item.block())
            .and_then(|kind| block_icons.as_deref().and_then(|icons| icons.get(kind)));
        let has_icon = icon_handle.is_some();
        let hovered = hover.entity == Some(entity);
        if hovered {
            hovered_item = item;
        }

        let selected_hotbar = slot.area == SlotArea::Hotbar && slot.index == placement.selected;
        if refresh_slots {
            update_slot_render(
                SlotRenderData {
                    item,
                    has_icon,
                    hovered,
                    selected_hotbar,
                },
                slot,
                children,
                &i18n,
                icon_handle,
                &mut node,
                &mut background,
                &mut border,
                &mut texts,
                &mut icons,
            );
        }
    }

    let Ok((mut tooltip_node, tooltip_children)) = tooltip.single_mut() else {
        return;
    };
    let Some(item) = hovered_item else {
        tooltip_node.display = Display::None;
        return;
    };
    let Ok(window) = windows.single() else {
        tooltip_node.display = Display::None;
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        tooltip_node.display = Display::None;
        return;
    };

    tooltip_node.display = Display::Flex;
    tooltip_node.left = Val::Px(cursor.x + 16.0);
    tooltip_node.top = Val::Px(cursor.y + 16.0);
    for child in tooltip_children.iter() {
        if let Ok(mut text) = texts.p1().get_mut(child) {
            text.0 = i18n.text(item.name_key());
        }
    }
}

fn update_slot_render(
    data: SlotRenderData,
    slot: &InventorySlot,
    children: &Children,
    i18n: &I18n,
    icon_handle: Option<Handle<Image>>,
    node: &mut Node,
    background: &mut BackgroundColor,
    border: &mut BorderColor,
    texts: &mut ParamSet<(
        Query<&mut Text, (Without<CarriedItemPreview>, Without<InventoryTooltip>)>,
        Query<&mut Text, (With<InventoryTooltip>, Without<CarriedItemPreview>)>,
    )>,
    icons: &mut Query<&mut ImageNode>,
) {
    node.display = if slot.area == SlotArea::Backpack && data.item.is_none() {
        Display::None
    } else {
        Display::Flex
    };

    let base_color = data
        .item
        .map(slot_color)
        .unwrap_or(Color::srgb(0.255, 0.251, 0.251));
    *background = if data.has_icon && data.hovered {
        Color::srgb(0.32, 0.31, 0.31).into()
    } else if data.has_icon {
        Color::srgb(0.255, 0.251, 0.251).into()
    } else if data.hovered && data.item.is_none() {
        Color::srgb(0.32, 0.31, 0.31).into()
    } else if data.hovered {
        base_color.with_alpha(1.0).into()
    } else {
        base_color.into()
    };
    *border = if data.selected_hotbar {
        BorderColor {
            top: Color::srgb(1.0, 0.94, 0.80),
            left: Color::srgb(1.0, 0.94, 0.80),
            right: Color::srgb(0.36, 0.25, 0.12),
            bottom: Color::srgb(0.36, 0.25, 0.12),
        }
    } else if data.hovered {
        hover_border()
    } else {
        inset_border()
    };

    for child in children.iter() {
        if let Ok(mut text) = texts.p0().get_mut(child) {
            text.0 = if data.has_icon {
                String::new()
            } else {
                data.item
                    .map(|kind| i18n.text(short_item_name(kind)))
                    .unwrap_or_default()
            };
        }
        if let Ok(mut image) = icons.get_mut(child) {
            *image = icon_handle.clone().map(ImageNode::new).unwrap_or_default();
        }
    }
}
