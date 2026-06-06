pub mod block_panels;
pub mod inventory;
pub mod menu;
pub mod save;
pub mod settings;

use bevy::prelude::*;

use save::text_prompt_input;
use settings::settings_menu_actions;

use crate::game::ui::core::host::{dispatch_ui_action, dispatch_ui_host_completions};

pub use block_panels::BlockPanelsPlugin;
pub use inventory::InventoryPlugin;
pub use menu::MenuPlugin;
pub use save::SavePlugin;
pub use settings::SettingsPlugin;

use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;

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
                dispatch_ui_action,
                dispatch_ui_host_completions,
                settings_menu_actions,
            )
                .chain()
                .in_set(UiAccessScope)
                .after(PerfScope::Input)
                .before(PerfScope::Menus),
        );
    }
}
