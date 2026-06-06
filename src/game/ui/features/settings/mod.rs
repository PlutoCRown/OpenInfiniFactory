mod actions;
pub mod types;
mod update;

use bevy::prelude::*;

pub use actions::settings_action_clicked;
pub(crate) use actions::settings_menu_actions;
pub use types::*;
pub use update::{
    update_settings_dropdowns_ui, update_settings_slider_drag_ui, update_settings_sliders_ui,
    update_settings_tabs_ui, update_settings_text_ui,
};

use crate::game::systems::perf::PerfScope;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SettingsTab::default())
            .insert_resource(OpenSettingsDropdown::default())
            .insert_resource(PendingKeyBind::default())
            .insert_resource(ActiveSettingsSlider::default())
            .add_observer(settings_action_clicked)
            .add_systems(
                Update,
                (
                    update_settings_text_ui,
                    (update_settings_sliders_ui, update_settings_slider_drag_ui).chain(),
                    update_settings_dropdowns_ui,
                    update_settings_tabs_ui,
                )
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            );
    }
}
