use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{GameMode, PlacementState, PlayingUiState, SolutionState};
use crate::game::ui::core::host::{UiAction, UiActionKind, UiHost, UiInstanceId};
use crate::game::ui::types::{CarriedItem, InventoryItems, InventorySlot, SlotArea};
use crate::shared::config::{ActionKeyName, GameConfig};

pub fn emit_inventory_slot_actions(
    mut click: On<Pointer<Click>>,
    mut writer: MessageWriter<UiAction>,
    slots: Query<&InventorySlot>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_host: Res<UiHost>,
) {
    if ui_host.modal_open() || *mode.get() != GameMode::Playing || !playing_ui.inventory_open {
        return;
    }
    let Ok(slot) = slots.get(click.entity) else {
        return;
    };
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
            &config,
            &mut inventory,
            &mut carried,
            &mut placement,
            &mut solution_state,
        );
    }
}

fn dispatch_inventory_slot_action(
    slot: InventorySlot,
    clicked_button: PointerButton,
    config: &GameConfig,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    solution_state: &mut SolutionState,
) {
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
        let before = inventory.hotbar;
        if let Some(item) = carried.take() {
            let previous = inventory.hotbar[slot.index].replace(item);
            carried.set(previous);
            placement.selected = slot.index;
        } else {
            if let Some(item) = clicked_item {
                inventory.hotbar[slot.index] = None;
                carried.set(Some(item));
            } else {
                carried.clear();
            }
            placement.selected = slot.index;
        }
        if inventory.hotbar != before {
            solution_state.dirty = true;
        }
    } else if let Some(item) = clicked_item {
        carried.set(Some(item));
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
