pub mod block_panels;
pub mod inventory;
pub mod menu;
pub mod save;
pub mod settings;

use bevy::prelude::*;

use block_panels::inline_text_edit_input;
use save::text_prompt_input;
use settings::settings_menu_actions;

use crate::game::ui::core::confirm_dialog::dispatch_confirm_completion;
use crate::game::ui::core::text_prompt::dispatch_text_prompt_completion;

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
                dispatch_text_prompt_completion,
                dispatch_confirm_completion,
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
                block_panels::update_active_block_panel,
                block_panels::update_block_panel_dropdowns,
            )
                .chain()
                .after(PerfScope::Animation)
                .before(PerfScope::Ui),
        );
    }
}
