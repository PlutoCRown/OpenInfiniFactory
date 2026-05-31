pub fn inventory_slot_clicks(
    mut interaction_query: Query<
        (&Interaction, &InventorySlot),
        (Changed<Interaction>, With<Button>),
    >,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
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

    for (interaction, slot) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let pick_button = config
            .input(ConfigAction::Pick)
            .mouse_button()
            .unwrap_or(MouseButton::Middle);
        if mouse_buttons.just_pressed(pick_button) {
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
            continue;
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
}

fn place_in_backpack(inventory: &mut InventoryItems, item: super::types::InventoryItem) -> bool {
    if inventory.backpack.iter().any(|slot| *slot == Some(item)) {
        return true;
    }
    if let Some(slot) = inventory.backpack.iter_mut().find(|slot| slot.is_none()) {
        *slot = Some(item);
        return true;
    }
    false
}
