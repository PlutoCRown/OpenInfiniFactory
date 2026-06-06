mod actions;
mod render;
mod types;

use bevy::prelude::*;

pub use actions::inventory_slot_clicks;
pub use render::{update_carried_item_ui, update_inventory_slots, update_inventory_title};
pub use types::InventoryTitleText;

use crate::game::systems::perf::PerfScope;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(inventory_slot_clicks)
            .add_systems(
                Update,
                (update_inventory_slots, update_carried_item_ui, update_inventory_title)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            );
    }
}
