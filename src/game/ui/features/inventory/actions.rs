use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{GameMode, PlacementState, PlayingUiState, SolutionState};
use crate::game::ui::core::host::{UiAction, UiActionKind, UiHost, UiInstanceId};
use crate::game::ui::{
    CarriedItem, HOTBAR_SLOTS, InlineTextEditState, InventoryItems, InventorySlot, PendingKeyBind,
    SlotArea, TextPromptState, UiRuntime,
};
use crate::shared::config::{ActionKeyName, GameConfig};

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

pub fn dispatch_inventory_slot_actions(
    mut actions: MessageReader<UiAction>,
    config: Res<GameConfig>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut solution_state: ResMut<SolutionState>,
    playing_ui: Res<PlayingUiState>,
) {
    for action in actions.read() {
        if action.instance != UiInstanceId::INVENTORY {
            continue;
        }
        let UiActionKind::InventorySlot { slot, button } = action.kind.clone() else {
            continue;
        };
        dispatch_inventory_slot_action(
            slot,
            button,
            playing_ui.inventory_open,
            &config,
            &mut inventory,
            &mut carried,
            &mut placement,
            &mut solution_state,
        );
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
    if clicked_button == pick_button {
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

/// 背包打开时操作快捷栏：有手持则覆盖放下，空手则拿起并清空该格
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
        placement.selected = index;
    } else if let Some(item) = inventory.hotbar[index] {
        inventory.hotbar[index] = None;
        carried.set(Some(item));
        placement.selected = index;
    } else {
        placement.selected = index;
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
