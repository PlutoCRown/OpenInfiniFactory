use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::state::{BuilderMode, PlacementState};
use crate::game::ui::access::{i18n, I18nRevision, UiMainThread};
use crate::game::ui::components::{hover_border, inset_border};
use crate::game::ui::types::{
    CarriedItem, CarriedItemPreview, InventoryItems, InventorySlot, InventoryTooltip, SlotArea,
    UiHoverState,
};
use crate::game::ui::widgets::{short_item_name, slot_color};
use crate::game::world::rendering::BlockIconAssets;

use super::types::InventoryTitleText;

fn builder_mode_name(mode: BuilderMode) -> String {
    i18n.t(match mode {
        BuilderMode::Edit => "mode.edit",
        BuilderMode::Play => "mode.play",
    })
}

pub fn update_inventory_title(
    _ui_thread: UiMainThread,
    builder_mode: Res<BuilderMode>,
    i18n_revision: Res<I18nRevision>,
    mut titles: Query<&mut Text, With<InventoryTitleText>>,
) {
    if !builder_mode.is_changed() && !i18n_revision.is_changed() {
        return;
    }
    for mut text in &mut titles {
        text.0 = i18n.fmt(
            "inventory.title",
            &[("mode", builder_mode_name(*builder_mode))],
        );
    }
}

pub fn update_inventory_slots(
    _ui_thread: UiMainThread,
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
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
    mut texts: ParamSet<(
        Query<&mut Text, Without<CarriedItemPreview>>,
        Query<&mut Text, Without<CarriedItemPreview>>,
    )>,
    mut icons: Query<&mut ImageNode>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut tooltip: Query<(&mut Node, &Children), (With<InventoryTooltip>, Without<Button>)>,
) {
    let mut hovered_item = None;
    for (entity, slot, children, mut node, mut background, mut border) in &mut slot_query {
        let item = match slot.area {
            SlotArea::Hotbar => inventory.hotbar[slot.index],
            SlotArea::Backpack => inventory.backpack[slot.index],
        };
        node.display = if slot.area == SlotArea::Backpack && item.is_none() {
            Display::None
        } else {
            Display::Flex
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
        let base_color = item
            .map(slot_color)
            .unwrap_or(Color::srgb(0.255, 0.251, 0.251));
        *background = if has_icon && hovered {
            Color::srgb(0.32, 0.31, 0.31).into()
        } else if has_icon {
            Color::srgb(0.255, 0.251, 0.251).into()
        } else if hovered && item.is_none() {
            Color::srgb(0.32, 0.31, 0.31).into()
        } else if hovered {
            base_color.with_alpha(1.0).into()
        } else {
            base_color.into()
        };
        *border = if selected_hotbar {
            BorderColor {
                top: Color::srgb(1.0, 0.94, 0.80),
                left: Color::srgb(1.0, 0.94, 0.80),
                right: Color::srgb(0.36, 0.25, 0.12),
                bottom: Color::srgb(0.36, 0.25, 0.12),
            }
        } else if hovered {
            hover_border()
        } else {
            inset_border()
        };

        for child in children.iter() {
            if let Ok(mut text) = texts.p0().get_mut(child) {
                text.0 = if has_icon {
                    String::new()
                } else {
                    item.map(|kind| i18n.t(short_item_name(kind)))
                        .unwrap_or_default()
                };
            }
            if let Ok(mut image) = icons.get_mut(child) {
                *image = icon_handle.clone().map(ImageNode::new).unwrap_or_default();
            }
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
            text.0 = i18n.t(item.name_key());
        }
    }
}

pub fn update_carried_item_ui(
    _ui_thread: UiMainThread,
    carried: Res<CarriedItem>,
    block_icons: Option<Res<BlockIconAssets>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut preview: Query<(&mut Node, &mut BackgroundColor, &Children), With<CarriedItemPreview>>,
    mut preview_images: Query<&mut ImageNode>,
    mut preview_text: Query<&mut Text>,
) {
    let Ok((mut style, mut background, children)) = preview.single_mut() else {
        return;
    };

    let Some(item) = carried.item() else {
        style.display = Display::None;
        for child in children.iter() {
            if let Ok(mut image) = preview_images.get_mut(child) {
                *image = ImageNode::default();
            }
            if let Ok(mut text) = preview_text.get_mut(child) {
                text.0.clear();
            }
        }
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        style.display = Display::None;
        return;
    };

    style.display = Display::Flex;
    style.left = Val::Px(cursor.x + 4.0);
    style.top = Val::Px(cursor.y + 4.0);
    *background = slot_color(item).with_alpha(0.9).into();

    let icon_handle = item
        .block()
        .and_then(|kind| block_icons.as_deref().and_then(|icons| icons.get(kind)));
    for child in children.iter() {
        if let Ok(mut image) = preview_images.get_mut(child) {
            *image = icon_handle
                .as_ref()
                .map(|handle| ImageNode::new(handle.clone()))
                .unwrap_or_default();
        }
        if let Ok(mut text) = preview_text.get_mut(child) {
            text.0.clear();
        }
    }
}
