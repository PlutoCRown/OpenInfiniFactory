use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{GameMode, PlacementState, SolutionState};
use crate::game::ui::types::{CarriedItem, InventoryItem, InventoryItems, InventorySlot, SlotArea};
use crate::shared::config::{ConfigAction, GameConfig};

pub fn inventory_slot_clicks(
    mut click: On<Pointer<Click>>,
    slots: Query<&InventorySlot>,
    config: Res<GameConfig>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut solution_state: ResMut<SolutionState>,
    mode: Res<GameMode>,
) {
    if *mode != GameMode::Inventory {
        return;
    }
    let Ok(slot) = slots.get(click.entity) else {
        return;
    };
    let clicked_button = click.event.button;
    click.propagate(false);

    let pick_button = config
        .input(ConfigAction::Pick)
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
        if let Some(item) = carried.item() {
            inventory.hotbar[slot.index] = Some(item);
            placement.selected = slot.index;
            carried.clear();
        } else {
            if let Some(item) = clicked_item {
                if place_in_backpack(&mut inventory, item) {
                    inventory.hotbar[slot.index] = None;
                    carried.clear();
                } else {
                    carried.set(Some(item));
                }
            } else {
                carried.clear();
            }
            placement.selected = slot.index;
        }
        if inventory.hotbar != before {
            solution_state.dirty = true;
        }
    } else {
        if let Some(item) = carried.take() {
            if inventory.backpack[slot.index].is_none() {
                inventory.backpack[slot.index] = Some(item);
            } else {
                let previous = inventory.backpack[slot.index].replace(item);
                carried.set(previous);
            }
        } else {
            carried.set(clicked_item);
        }
    }
    placement.selection.clear();
    placement.edit_gesture = None;
}

fn place_in_backpack(inventory: &mut InventoryItems, item: InventoryItem) -> bool {
    if inventory.backpack.iter().any(|slot| *slot == Some(item)) {
        return true;
    }
    if let Some(slot) = inventory.backpack.iter_mut().find(|slot| slot.is_none()) {
        *slot = Some(item);
        return true;
    }
    false
}

fn pointer_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => PointerButton::Primary,
    }
}
