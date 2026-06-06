pub use crate::game::ui::systems::inventory_slot_clicks;
pub use crate::game::ui::systems::update_carried_item_ui;
pub use crate::game::ui::systems::update_inventory_slots;

use bevy::prelude::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(inventory_slot_clicks);
    }
}
