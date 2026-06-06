mod actions;
pub mod types;
mod update;

use bevy::prelude::*;

pub use actions::emit_settings_actions;
pub(crate) use actions::{dispatch_settings_actions, settings_menu_actions};
pub use types::*;
pub use update::{
    update_settings_dropdowns_ui, update_settings_slider_drag_ui, update_settings_sliders_ui,
    update_settings_tabs_ui, update_settings_text_ui,
};

use crate::game::ui::access::UiAccessScope;

use crate::game::systems::perf::PerfScope;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SettingsTab::default())
            .insert_resource(OpenSettingsDropdown::default())
            .insert_resource(PendingKeyBind::default())
            .insert_resource(ActiveSettingsSlider::default())
            .add_observer(emit_settings_actions)
            .add_systems(
                Update,
                dispatch_settings_actions
                    .in_set(UiAccessScope)
                    .after(PerfScope::Input)
                    .before(PerfScope::Menus),
            )
            .add_systems(
                Update,
                (
                    update_settings_text_ui,
                    (update_settings_sliders_ui, update_settings_slider_drag_ui).chain(),
                    update_settings_dropdowns_ui,
                    update_settings_tabs_ui,
                )
                    .in_set(UiAccessScope)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            );
    }
}
