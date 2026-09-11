pub mod block_panels;
pub mod gameplay_toast;
pub mod inventory;
pub mod pause_menu;
pub mod playing_overlays;
pub mod save;
pub mod save_settings;
pub mod session_busy;
pub mod settings;
pub mod start_menu;
pub mod start_menu_mounts;
pub mod tutorial;
pub mod virtual_remote;

use bevy::prelude::*;

use crate::game::ui::core::text_prompt::text_prompt_hotkeys;
use settings::settings_menu_actions;

use crate::game::ui::core::host::{dispatch_ui_action, dispatch_ui_host_completions};

pub use block_panels::BlockPanelsPlugin;
pub use gameplay_toast::{GameplayToast, GameplayToastPlugin};
pub use inventory::InventoryPlugin;
pub use pause_menu::PauseMenuPlugin;
pub use save::SavePlugin;
pub use save_settings::SaveSettingsPlugin;
pub use session_busy::SessionBusyUiPlugin;
pub use settings::SettingsPlugin;
pub use start_menu::StartMenuPlugin;
pub use tutorial::{
    TutorialCatalog, TutorialDefinition, TutorialIntent, TutorialPlugin, TutorialStep,
};
pub use virtual_remote::VirtualRemotePlugin;

pub struct UiFeaturesPlugin;

impl Plugin for UiFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            StartMenuPlugin,
            PauseMenuPlugin,
            SavePlugin,
            SaveSettingsPlugin,
            SettingsPlugin,
            BlockPanelsPlugin,
            InventoryPlugin,
            GameplayToastPlugin,
            SessionBusyUiPlugin,
            VirtualRemotePlugin,
            TutorialPlugin,
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
                .in_set(crate::game::schedule::GameSet::Menus),
        );
    }
}
