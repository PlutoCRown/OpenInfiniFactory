pub mod block_panels;
pub mod inventory;
pub mod pause_menu;
pub mod playing_overlays;
pub mod save;
pub mod session_busy;
pub mod settings;
pub mod start_menu;
pub mod start_menu_mounts;
pub mod virtual_remote;

use bevy::prelude::*;

use crate::game::ui::core::text_prompt::text_prompt_hotkeys;
use settings::settings_menu_actions;

use crate::game::ui::core::host::{dispatch_ui_action, dispatch_ui_host_completions};

pub use block_panels::BlockPanelsPlugin;
pub use inventory::InventoryPlugin;
pub use pause_menu::PauseMenuPlugin;
pub use playing_overlays::PlayingOverlaysPlugin;
pub use start_menu::StartMenuPlugin;
pub use start_menu_mounts::StartMenuMountsPlugin;
pub use save::SavePlugin;
pub use session_busy::SessionBusyUiPlugin;
pub use settings::SettingsPlugin;
pub use virtual_remote::VirtualRemotePlugin;

use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;

pub struct UiFeaturesPlugin;

impl Plugin for UiFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            StartMenuPlugin,
            StartMenuMountsPlugin,
            PauseMenuPlugin,
            PlayingOverlaysPlugin,
            SavePlugin,
            SettingsPlugin,
            BlockPanelsPlugin,
            InventoryPlugin,
            SessionBusyUiPlugin,
            VirtualRemotePlugin,
        ))
        .add_systems(
            Update,
            (
                text_prompt_hotkeys,
                dispatch_ui_action,
                dispatch_ui_host_completions,
                settings_menu_actions,
            )
                .chain()
                .in_set(UiAccessScope)
                .after(PerfScope::Placement)
                .before(PerfScope::Menus),
        );
    }
}
