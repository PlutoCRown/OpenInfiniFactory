mod actions;
mod render;
mod types;

use bevy::prelude::*;

pub use actions::{dispatch_inventory_slot_actions, emit_inventory_slot_actions};
pub use render::{update_carried_item_ui, update_inventory_slots, update_inventory_title};
pub use types::InventoryTitleText;

use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_inventory_slot_actions)
            .add_systems(
                Update,
                dispatch_inventory_slot_actions
                    .after(PerfScope::Input)
                    .before(PerfScope::Menus),
            )
            .add_systems(
                Update,
                (update_inventory_slots, update_carried_item_ui, update_inventory_title)
                    .in_set(UiAccessScope)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            );
    }
}
