mod components;
pub mod core;
pub mod features;
mod layout;
mod screens;
mod systems;
mod types;
mod widgets;

use bevy::prelude::*;

pub use layout::{setup_menu_ui, setup_playing_ui_system};
pub use systems::{
    apply_ui_font, load_ui_font, panel_close_clicked, panel_drag_ended, panel_drag_started,
    panel_dragged, ui_hovered, ui_unhovered, update_confirm_dialog_ui, update_hud_visibility,
    update_localized_ui, update_panel_visibility, update_save_list_ui, update_settings_dropdowns_ui,
    update_settings_slider_drag_ui, update_settings_sliders_ui, update_settings_tabs_ui,
    update_settings_text_ui, update_status_ui, update_text_prompt_ui, update_ui_layers,
};
pub use types::*;

use crate::game::systems::perf::PerfScope;
use components::{
    button_hovered, button_pressed, button_released, button_unhovered, update_scroll_containers,
};
use features::UiFeaturesPlugin;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiRuntime::default())
            .insert_resource(CarriedItem::default())
            .insert_resource(PanelDragState::default())
            .insert_resource(UiHoverState::default())
            .add_plugins(UiFeaturesPlugin)
            .add_observer(panel_close_clicked)
            .add_observer(panel_drag_started)
            .add_observer(panel_dragged)
            .add_observer(panel_drag_ended)
            .add_observer(button_hovered)
            .add_observer(button_unhovered)
            .add_observer(button_pressed)
            .add_observer(button_released)
            .add_observer(ui_hovered)
            .add_observer(ui_unhovered)
            .add_systems(
                Update,
                (
                    update_localized_ui,
                    update_settings_text_ui,
                    (update_settings_sliders_ui, update_settings_slider_drag_ui).chain(),
                    update_settings_dropdowns_ui,
                    update_settings_tabs_ui,
                    update_scroll_containers,
                    update_confirm_dialog_ui,
                    update_text_prompt_ui,
                    apply_ui_font,
                )
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            )
            .add_systems(
                Update,
                (
                    (update_panel_visibility, update_ui_layers).chain(),
                    update_save_list_ui,
                )
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            )
            .add_systems(
                Update,
                (update_status_ui, update_hud_visibility)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            );
    }
}
