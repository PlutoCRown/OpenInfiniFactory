mod actions;
pub mod types;

use bevy::prelude::*;

pub use actions::settings_action_clicked;
pub(crate) use actions::settings_menu_actions;
pub use types::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SettingsTab::default())
            .insert_resource(OpenSettingsDropdown::default())
            .insert_resource(PendingKeyBind::default())
            .insert_resource(ActiveSettingsSlider::default())
            .add_observer(settings_action_clicked);
    }
}
