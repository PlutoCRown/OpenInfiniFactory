mod actions;
mod render;
mod types;

use bevy::prelude::*;

use actions::{
    dispatch_inventory_slot_actions, emit_inventory_slot_actions, emit_inventory_tab_actions,
    inventory_hotbar_digit_input, reset_closed_touch_inventory, touch_inventory_drag_ended,
    touch_inventory_drag_started, touch_inventory_dragged,
};
use render::{
    update_carried_item_ui, update_inventory_slots, update_inventory_tabs, update_inventory_title,
    update_item_tooltip,
};
pub use types::{InventoryTabButton, InventoryTitleText, TouchInventoryState};

use crate::game::state::GameMode;
use crate::game::ui::core::UiNavigation;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<crate::game::ui::core::host::UiAction<actions::InventoryAction>>();
        app.init_resource::<TouchInventoryState>()
            .add_observer(actions::return_carried_item_on_close)
            .add_observer(emit_inventory_slot_actions)
            .add_observer(emit_inventory_tab_actions)
            .add_observer(touch_inventory_drag_started)
            .add_observer(touch_inventory_dragged)
            .add_observer(touch_inventory_drag_ended)
            .add_systems(
                Update,
                (
                    reset_closed_touch_inventory,
                    inventory_hotbar_digit_input,
                    dispatch_inventory_slot_actions,
                )
                    .in_set(crate::game::schedule::GameSet::Menus),
            )
            // 热栏 / tooltip 常驻，不能绑 inventory_open，否则关背包后不刷新、tooltip 残留
            .add_systems(
                Update,
                (
                    update_inventory_slots,
                    update_item_tooltip,
                    update_carried_item_ui,
                )
                    .run_if(|mode: Res<State<GameMode>>| *mode.get() == GameMode::Playing)
                    .in_set(crate::game::schedule::GameSet::UiInventory),
            )
            .add_systems(
                Update,
                (update_inventory_title, update_inventory_tabs)
                    .run_if(|ui_navigation: Res<UiNavigation>| ui_navigation.is_inventory_open())
                    .in_set(crate::game::schedule::GameSet::UiInventory),
            );
    }
}
