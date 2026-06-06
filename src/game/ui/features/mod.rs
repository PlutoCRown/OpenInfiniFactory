pub mod block_panels;
pub mod inventory;
pub mod menu;
pub mod save;
pub mod settings;

use bevy::prelude::*;

use block_panels::{
    inline_text_edit_input, update_active_block_panel, update_block_panel_dropdowns,
};
use save::text_prompt_input;
use settings::settings_menu_actions;

pub use block_panels::BlockPanelsPlugin;
pub use inventory::InventoryPlugin;
pub use menu::MenuPlugin;
pub use save::SavePlugin;
pub use settings::SettingsPlugin;

use crate::game::systems::perf::PerfScope;

pub struct UiFeaturesPlugin;

impl Plugin for UiFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MenuPlugin,
            SavePlugin,
            SettingsPlugin,
            BlockPanelsPlugin,
            InventoryPlugin,
        ))
        .add_systems(
            Update,
            (
                text_prompt_input,
                settings_menu_actions,
                inline_text_edit_input,
            )
                .chain()
                .after(PerfScope::Input)
                .before(PerfScope::Menus),
        )
        .add_systems(
            Update,
            (
                (update_active_block_panel, update_block_panel_dropdowns).chain(),
                inventory::update_inventory_slots,
                inventory::update_carried_item_ui,
            )
                .after(PerfScope::Animation)
                .before(PerfScope::Ui),
        );
    }
}
