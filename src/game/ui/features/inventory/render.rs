use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::state::{BuilderMode, PlacementState, SolutionState, WorldEntryMode};
use crate::game::ui::access::{UiMainThread, i18n};
use crate::game::ui::components::{default_button_size, hover_border, inset_border};
use crate::game::ui::features::inventory::InventoryTabButton;
use crate::game::ui::types::{
    AreaKind, CarriedItem, CarriedItemPreview, FreeInventoryTab, HoverTooltip, InventoryItem,
    InventoryItems, InventorySlot, ItemTooltip, ItemTooltipDescription, ItemTooltipName, SlotArea,
};
use crate::game::ui::widgets::{short_item_name, slot_color};
use crate::game::world::rendering::BlockIconAssets;

use super::types::InventoryTitleText;

fn builder_mode_key(mode: BuilderMode) -> &'static str {
    match mode {
        BuilderMode::Edit => "mode.edit",
        BuilderMode::Play => "mode.play",
    }
}

pub fn update_inventory_title(
    _ui_thread: UiMainThread,
    builder_mode: Res<BuilderMode>,
    solution_state: Res<SolutionState>,
    mut titles: Query<&mut Text, With<InventoryTitleText>>,
    added: Query<(), Added<InventoryTitleText>>,
) {
    if !builder_mode.is_changed() && !solution_state.is_changed() && added.is_empty() {
        return;
    }
    let mode = i18n.t(match solution_state.entry {
        WorldEntryMode::Free => "save.kind.free",
        _ => builder_mode_key(*builder_mode),
    });
    for mut text in &mut titles {
        text.0 = i18n.fmt("inventory.title", &[("mode", mode.as_str())]);
    }
}

/// Free 背包页签选中样式
pub fn update_inventory_tabs(
    _ui_thread: UiMainThread,
    free_tab: Res<FreeInventoryTab>,
    solution_state: Res<SolutionState>,
    mut tabs: Query<(&InventoryTabButton, &mut BackgroundColor, &mut BorderColor)>,
    added: Query<(), Added<InventoryTabButton>>,
) {
    if solution_state.entry != WorldEntryMode::Free {
        return;
    }
    if !free_tab.is_changed() && added.is_empty() {
        return;
    }
    for (button, mut background, mut border) in &mut tabs {
        let selected = button.0 == *free_tab;
        *background = if selected {
            Color::srgba(0.22, 0.35, 0.32, 0.96).into()
        } else {
            Color::srgb(0.22, 0.24, 0.26).into()
        };
        *border = if selected {
            BorderColor::all(Color::srgb(1.0, 0.94, 0.80))
        } else {
            BorderColor::all(Color::srgb(0.4, 0.42, 0.45))
        };
    }
}

/// 物品格内容与选中/悬停样式：只在相关状态变化时刷新，不跟鼠标每帧重绘
pub fn update_inventory_slots(
    _ui_thread: UiMainThread,
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    block_icons: Option<Res<BlockIconAssets>>,
    mut commands: Commands,
    mut initialized: Local<bool>,
    mut last_selected: Local<usize>,
    mut had_block_icons: Local<bool>,
    mut last_slot_count: Local<usize>,
    mut last_hovered: Local<Option<Entity>>,
    mut slot_query: Query<
        (
            Entity,
            &InventorySlot,
            &Interaction,
            &Children,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&HoverTooltip>,
        ),
        (With<Button>, Without<ItemTooltip>),
    >,
    mut slot_texts: Query<
        &mut Text,
        (
            Without<CarriedItemPreview>,
            Without<ItemTooltipName>,
            Without<ItemTooltipDescription>,
        ),
    >,
    mut icons: Query<&mut ImageNode>,
) {
    let icons_ready = block_icons.is_some();
    let icons_became_ready = icons_ready && !*had_block_icons;
    *had_block_icons = icons_ready;
    let icons_changed = block_icons.as_ref().is_some_and(|icons| icons.is_changed());
    // PlacementState.target 几乎每帧变化，只跟踪快捷栏选中下标
    let selected_changed = !*initialized || placement.selected != *last_selected;
    let inventory_changed = !*initialized || inventory.is_changed();
    let slot_count = slot_query.iter().len();
    // 背包按需挂载后 Slot 实体会增减，必须重新灌内容
    let slots_changed = !*initialized || slot_count != *last_slot_count;

    let hovered_entity = slot_query.iter().find_map(|(entity, _, interaction, ..)| {
        (*interaction == Interaction::Hovered).then_some(entity)
    });
    let hover_changed = !*initialized || hovered_entity != *last_hovered;

    if !inventory_changed
        && !selected_changed
        && !hover_changed
        && !icons_changed
        && !icons_became_ready
        && !slots_changed
    {
        return;
    }
    *initialized = true;
    *last_selected = placement.selected;
    *last_hovered = hovered_entity;
    *last_slot_count = slot_count;

    let refresh_content =
        inventory_changed || icons_changed || icons_became_ready || slots_changed;

    for (entity, slot, interaction, children, mut node, mut background, mut border, tip) in
        &mut slot_query
    {
        let item = match slot.area {
            SlotArea::Hotbar => inventory.hotbar[slot.index],
            SlotArea::Backpack => inventory.backpack[slot.index],
        };
        let next = if slot.area == SlotArea::Backpack && item.is_none() {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != next {
            node.display = next;
        }

        let icon_handle = item.and_then(|item| match item {
            InventoryItem::Block(kind) => block_icons.as_deref().and_then(|icons| icons.get(kind)),
            InventoryItem::Area(AreaKind::Selection) => {
                block_icons.as_deref().and_then(|icons| icons.selection())
            }
            InventoryItem::LightPanel => {
                block_icons.as_deref().and_then(|icons| icons.light_panel())
            }
        });
        let has_icon = icon_handle.is_some();
        let hovered = *interaction == Interaction::Hovered;
        let selected_hotbar = slot.area == SlotArea::Hotbar && slot.index == placement.selected;

        *background = slot_background(item, has_icon, hovered);
        *border = slot_border(selected_hotbar, hovered);

        let next_tip = item.map(|item| HoverTooltip {
            name_key: item.name_key(),
            description_key: item.description_key(),
        });
        match (tip.copied(), next_tip) {
            (Some(old), Some(new)) if old != new => {
                commands.entity(entity).insert(new);
            }
            (None, Some(new)) => {
                commands.entity(entity).insert(new);
            }
            (Some(_), None) => {
                commands.entity(entity).remove::<HoverTooltip>();
            }
            _ => {}
        }

        if !refresh_content {
            continue;
        }
        for child in children.iter() {
            if let Ok(mut text) = slot_texts.get_mut(child) {
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
}

/// 通用悬停提示：任意挂了 HoverTooltip 的按钮；手持物品时隐藏
pub fn update_item_tooltip(
    _ui_thread: UiMainThread,
    carried: Res<CarriedItem>,
    targets: Query<(&HoverTooltip, &Interaction)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut last_keys: Local<Option<HoverTooltip>>,
    mut tooltip: Query<(&mut Node, &mut Visibility), (With<ItemTooltip>, Without<Button>)>,
    mut tooltip_name: Query<
        &mut Text,
        (With<ItemTooltipName>, Without<ItemTooltipDescription>),
    >,
    mut tooltip_desc: Query<
        &mut Text,
        (With<ItemTooltipDescription>, Without<ItemTooltipName>),
    >,
) {
    let Ok((mut tooltip_node, mut tooltip_visibility)) = tooltip.single_mut() else {
        return;
    };

    // 手里拿着东西时不显示，避免和悬浮 Icon 叠在一起
    if carried.item().is_some() {
        if tooltip_node.display != Display::None {
            tooltip_node.display = Display::None;
        }
        tooltip_visibility.set_if_neq(Visibility::Hidden);
        *last_keys = None;
        return;
    }

    let hovered = targets.iter().find_map(|(tip, interaction)| {
        (*interaction == Interaction::Hovered).then_some(*tip)
    });

    let Some(tip) = hovered else {
        if tooltip_node.display != Display::None {
            tooltip_node.display = Display::None;
        }
        tooltip_visibility.set_if_neq(Visibility::Hidden);
        *last_keys = None;
        return;
    };
    let Ok(window) = windows.single() else {
        tooltip_node.display = Display::None;
        tooltip_visibility.set_if_neq(Visibility::Hidden);
        *last_keys = None;
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        tooltip_node.display = Display::None;
        tooltip_visibility.set_if_neq(Visibility::Hidden);
        *last_keys = None;
        return;
    };

    tooltip_visibility.set_if_neq(Visibility::Visible);
    let was_hidden = tooltip_node.display == Display::None;
    tooltip_node.display = Display::Flex;
    tooltip_node.left = Val::Px(cursor.x + 16.0);
    tooltip_node.top = Val::Px(cursor.y + 16.0);
    if was_hidden || last_keys.as_ref() != Some(&tip) {
        if let Ok(mut text) = tooltip_name.single_mut() {
            text.0 = i18n.t(tip.name_key);
        }
        if let Ok(mut text) = tooltip_desc.single_mut() {
            text.0 = i18n.t(tip.description_key);
        }
        *last_keys = Some(tip);
    }
}

/// 手持物品预览：纯 Icon 居中跟光标，图标只在手持变化时更新
pub fn update_carried_item_ui(
    _ui_thread: UiMainThread,
    carried: Res<CarriedItem>,
    block_icons: Option<Res<BlockIconAssets>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut preview: Query<(&mut Node, &mut ImageNode), With<CarriedItemPreview>>,
) {
    let Ok((mut style, mut image)) = preview.single_mut() else {
        return;
    };

    let Some(item) = carried.item() else {
        if style.display != Display::None {
            style.display = Display::None;
            *image = ImageNode::default();
        }
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        if style.display != Display::None {
            style.display = Display::None;
        }
        return;
    };

    // 图标中心对准光标，避免偏右下显得「不跟手」
    let half = default_button_size(46.0) * 0.5;
    style.display = Display::Flex;
    style.left = Val::Px(cursor.x - half);
    style.top = Val::Px(cursor.y - half);

    if !carried.is_changed() && !block_icons.as_ref().is_some_and(|icons| icons.is_changed()) {
        return;
    }

    let icon_handle = match item {
        InventoryItem::Block(kind) => block_icons.as_deref().and_then(|icons| icons.get(kind)),
        InventoryItem::Area(AreaKind::Selection) => {
            block_icons.as_deref().and_then(|icons| icons.selection())
        }
        InventoryItem::LightPanel => {
            block_icons.as_deref().and_then(|icons| icons.light_panel())
        }
    };
    *image = icon_handle
        .map(|handle| ImageNode::new(handle.clone()))
        .unwrap_or_default();
}

fn slot_background(item: Option<InventoryItem>, has_icon: bool, hovered: bool) -> BackgroundColor {
    let base_color = item
        .map(slot_color)
        .unwrap_or(Color::srgb(0.255, 0.251, 0.251));
    if has_icon && hovered {
        Color::srgb(0.32, 0.31, 0.31).into()
    } else if has_icon {
        Color::srgb(0.255, 0.251, 0.251).into()
    } else if hovered && item.is_none() {
        Color::srgb(0.32, 0.31, 0.31).into()
    } else if hovered {
        base_color.with_alpha(1.0).into()
    } else {
        base_color.into()
    }
}

fn slot_border(selected_hotbar: bool, hovered: bool) -> BorderColor {
    if selected_hotbar {
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
    }
}
