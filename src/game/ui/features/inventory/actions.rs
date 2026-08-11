use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Drag, DragEnd, DragStart, Pointer};
use bevy::prelude::*;

use crate::game::state::{GameMode, PlacementState, PlayingUiState, SolutionState, WorldEntryMode};
use crate::game::ui::components::ui_logical_bounds;
use crate::game::ui::core::host::{UiAction, UiActionKind, UiHost, UiInstanceId};
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::inventory::InventoryTabButton;
use crate::game::ui::types::FreeInventoryTab;
use crate::game::ui::{
    CarriedItem, HOTBAR_SLOTS, InlineTextEditState, InventoryItems, InventorySlot, PendingKeyBind,
    SlotArea, TextPromptState, UiRuntime,
};
use crate::shared::config::{ActionKeyName, GameConfig};
use crate::shared::touch_profile::TouchProfile;

use super::types::TouchInventoryState;

pub fn emit_inventory_slot_actions(
    mut click: On<Pointer<Click>>,
    mut writer: MessageWriter<UiAction>,
    slots: Query<&InventorySlot>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_host: Res<UiHost>,
) {
    if ui_host.modal_open() || *mode.get() != GameMode::Playing {
        return;
    }
    let Ok(slot) = slots.get(click.entity) else {
        return;
    };
    // 背包关闭时只接受快捷栏点击（触屏切换选中）
    if !playing_ui.inventory_open && slot.area != SlotArea::Hotbar {
        return;
    }
    let button = click.event.button;
    click.propagate(false);
    writer.write(UiAction {
        instance: UiInstanceId::INVENTORY,
        kind: UiActionKind::InventorySlot {
            slot: *slot,
            button,
        },
    });
}

/// Free 背包页签点击
pub fn emit_inventory_tab_actions(
    mut click: On<Pointer<Click>>,
    mut writer: MessageWriter<UiAction>,
    tabs: Query<&InventoryTabButton>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    solution_state: Res<SolutionState>,
    ui_host: Res<UiHost>,
) {
    if ui_host.modal_open()
        || *mode.get() != GameMode::Playing
        || !playing_ui.inventory_open
        || solution_state.entry != WorldEntryMode::Free
        || !primary_click(&mut click)
    {
        return;
    }
    let Ok(tab) = tabs.get(click.entity) else {
        return;
    };
    click.propagate(false);
    writer.write(UiAction {
        instance: UiInstanceId::INVENTORY,
        kind: UiActionKind::InventoryTab(tab.0),
    });
}

pub fn dispatch_inventory_slot_actions(
    mut actions: MessageReader<UiAction>,
    config: Res<GameConfig>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut solution_state: ResMut<SolutionState>,
    mut free_tab: ResMut<FreeInventoryTab>,
    touch: Res<TouchProfile>,
    mut touch_inventory: ResMut<TouchInventoryState>,
    playing_ui: Res<PlayingUiState>,
) {
    for action in actions.read() {
        if action.instance != UiInstanceId::INVENTORY {
            continue;
        }
        match action.kind.clone() {
            UiActionKind::InventoryTab(tab) => {
                if solution_state.entry != WorldEntryMode::Free || *free_tab == tab {
                    continue;
                }
                *free_tab = tab;
                inventory.fill_free_backpack(tab);
                touch_inventory.clear();
            }
            UiActionKind::InventorySlot { slot, button } => {
                dispatch_inventory_slot_action(
                    slot,
                    button,
                    playing_ui.inventory_open,
                    &config,
                    &mut inventory,
                    &mut carried,
                    &mut placement,
                    &mut solution_state,
                    *touch,
                    &mut touch_inventory,
                );
            }
            _ => {}
        }
    }
}

/// 触控拖动从背包目录拿起物品，并关闭点选 tooltip
pub fn touch_inventory_drag_started(
    mut drag_start: On<Pointer<DragStart>>,
    touch: Res<TouchProfile>,
    playing_ui: Res<PlayingUiState>,
    ui_host: Res<UiHost>,
    slots: Query<(&InventorySlot, &ComputedNode, &UiGlobalTransform)>,
    inventory: Res<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut touch_inventory: ResMut<TouchInventoryState>,
) {
    if !touch.enabled
        || !playing_ui.inventory_open
        || ui_host.modal_open()
        || drag_start.event.button != PointerButton::Primary
    {
        return;
    }
    let Ok((slot, computed, transform)) = slots.get(drag_start.entity) else {
        return;
    };
    if slot.area != SlotArea::Backpack {
        return;
    }
    let Some(item) = inventory.backpack[slot.index] else {
        return;
    };
    drag_start.propagate(false);
    touch_inventory.selected_backpack = None;
    let pointer = drag_start.pointer_location.position;
    let slot_center = ui_logical_bounds(computed, transform).center();
    touch_inventory.drag_pointer = Some(slot_center);
    touch_inventory.drag_pointer_id = Some(drag_start.pointer_id);
    touch_inventory.drag_grab_offset = pointer - slot_center;
    touch_inventory.dragging = true;
    carried.set(Some(item));
}

/// 触控拖动时记录手指位置，让手持物品图标跟随手指
pub fn touch_inventory_dragged(
    mut drag: On<Pointer<Drag>>,
    touch: Res<TouchProfile>,
    mut touch_inventory: ResMut<TouchInventoryState>,
) {
    if !touch.enabled || !touch_inventory.dragging || drag.event.button != PointerButton::Primary {
        return;
    }
    drag.propagate(false);
    touch_inventory.drag_pointer =
        Some(drag.pointer_location.position - touch_inventory.drag_grab_offset);
}

/// 触控拖动结束时放入手指下的快捷栏，否则取消手持
pub fn touch_inventory_drag_ended(
    mut drag_end: On<Pointer<DragEnd>>,
    touch: Res<TouchProfile>,
    slots: Query<(&InventorySlot, &Node, &ComputedNode, &UiGlobalTransform)>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut solution_state: ResMut<SolutionState>,
    mut touch_inventory: ResMut<TouchInventoryState>,
) {
    if !touch.enabled
        || !touch_inventory.dragging
        || drag_end.event.button != PointerButton::Primary
    {
        return;
    }
    drag_end.propagate(false);
    let pointer = drag_end.pointer_location.position;
    let target = slots
        .iter()
        .filter(|(slot, node, ..)| slot.area == SlotArea::Hotbar && node.display != Display::None)
        .find_map(|(slot, _, computed, transform)| {
            let bounds = ui_logical_bounds(computed, transform);
            (pointer.x >= bounds.min.x
                && pointer.x <= bounds.max.x
                && pointer.y >= bounds.min.y
                && pointer.y <= bounds.max.y)
                .then_some(slot.index)
        });
    let item = carried.take();
    if let (Some(index), Some(item)) = (target, item) {
        if inventory.hotbar[index] != Some(item) {
            inventory.hotbar[index] = Some(item);
            solution_state.dirty = true;
        }
    }
    touch_inventory.clear();
    placement.selection.clear();
    placement.edit_gesture = None;
}

/// 背包关闭后清除触控点选和拖动状态，避免下次打开残留
pub fn reset_closed_touch_inventory(
    touch: Res<TouchProfile>,
    playing_ui: Res<PlayingUiState>,
    mut carried: ResMut<CarriedItem>,
    mut touch_inventory: ResMut<TouchInventoryState>,
) {
    if touch.enabled
        && !playing_ui.inventory_open
        && (touch_inventory.selected_backpack.is_some() || touch_inventory.dragging)
    {
        carried.clear();
        touch_inventory.clear();
    }
}

/// 背包打开时数字键：有手持放到对应快捷栏，空手则拿起该格物品
pub fn inventory_hotbar_digit_input(
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    text_prompt: Res<TextPromptState>,
    pending_key_bind: Res<PendingKeyBind>,
    inline_edit: Res<InlineTextEditState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut solution_state: ResMut<SolutionState>,
) {
    if *mode.get() != GameMode::Playing || !playing_ui.inventory_open {
        return;
    }
    let typing = pending_key_bind.0.is_some() || text_prompt.is_open() || inline_edit.is_active();
    if typing || ui_runtime.blocks_gameplay() {
        return;
    }

    for (key, index) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ] {
        if keys.just_pressed(key) && index < HOTBAR_SLOTS {
            apply_open_inventory_hotbar(
                index,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut solution_state,
            );
        }
    }
}

fn dispatch_inventory_slot_action(
    slot: InventorySlot,
    clicked_button: PointerButton,
    inventory_open: bool,
    config: &GameConfig,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    solution_state: &mut SolutionState,
    touch: TouchProfile,
    touch_inventory: &mut TouchInventoryState,
) {
    // 背包关闭：快捷栏左键只切换当前选中格（等同数字键）
    if !inventory_open {
        if slot.area == SlotArea::Hotbar && clicked_button == PointerButton::Primary {
            if placement.selected != slot.index {
                placement.selection.clear();
                placement.edit_gesture = None;
                placement.selected = slot.index;
            }
        }
        return;
    }

    let pick_button = config
        .input(ActionKeyName::Pick)
        .mouse_button()
        .map(pointer_button)
        .unwrap_or(PointerButton::Middle);
    if !touch.enabled && clicked_button == pick_button {
        if slot.area == SlotArea::Hotbar {
            if inventory.hotbar[slot.index].is_some() {
                inventory.hotbar[slot.index] = None;
                solution_state.dirty = true;
            }
            if placement.selected == slot.index {
                carried.clear();
            }
        }
        placement.selection.clear();
        placement.edit_gesture = None;
        return;
    }

    if clicked_button != PointerButton::Primary {
        return;
    }

    let clicked_item = match slot.area {
        SlotArea::Hotbar => inventory.hotbar[slot.index],
        SlotArea::Backpack => inventory.backpack[slot.index],
    };

    if touch.enabled {
        match slot.area {
            SlotArea::Backpack => {
                carried.clear();
                touch_inventory.selected_backpack = clicked_item.map(|item| (slot.index, item));
            }
            SlotArea::Hotbar => {
                if let Some((_, item)) = touch_inventory.selected_backpack.take() {
                    if inventory.hotbar[slot.index] != Some(item) {
                        inventory.hotbar[slot.index] = Some(item);
                        solution_state.dirty = true;
                    }
                } else if placement.selected != slot.index {
                    placement.selected = slot.index;
                }
            }
        }
        placement.selection.clear();
        placement.edit_gesture = None;
        return;
    }

    if slot.area == SlotArea::Hotbar {
        apply_open_inventory_hotbar(slot.index, inventory, carried, placement, solution_state);
        return;
    }

    if carried.item().is_some() {
        // 背包格：有手持则取消手里的东西
        carried.clear();
    } else if let Some(item) = clicked_item {
        // 背包格：空手则拿起（目录不移除）
        carried.set(Some(item));
    }
    placement.selection.clear();
    placement.edit_gesture = None;
}

/// 背包打开时操作快捷栏：有手持则覆盖放下，空手则拿起并清空该格（不切换当前选中）
fn apply_open_inventory_hotbar(
    index: usize,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    solution_state: &mut SolutionState,
) {
    let before = inventory.hotbar;
    if let Some(item) = carried.take() {
        inventory.hotbar[index] = Some(item);
    } else if let Some(item) = inventory.hotbar[index] {
        inventory.hotbar[index] = None;
        carried.set(Some(item));
    }
    if inventory.hotbar != before {
        solution_state.dirty = true;
    }
    placement.selection.clear();
    placement.edit_gesture = None;
}

fn pointer_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => PointerButton::Primary,
    }
}
