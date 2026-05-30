mod components;
mod layout;
mod systems;
mod theme;
mod types;
mod widgets;

use bevy::prelude::*;

pub use crate::game::state::UiPanelId;
pub use layout::setup_ui;
pub use systems::{
    apply_ui_font, inventory_slot_clicks, load_ui_font, update_button_hover_ui,
    update_carried_item_ui, update_confirm_dialog_ui, update_converter_ui, update_generator_ui,
    update_hud_visibility, update_inventory_slots, update_labeler_ui, update_localized_ui,
    update_panel_visibility, update_save_list_ui, update_scroll_containers,
    update_settings_dropdowns_ui, update_settings_slider_drag_ui, update_settings_sliders_ui,
    update_settings_tabs_ui, update_settings_text_ui, update_status_ui, update_teleport_ui,
};
pub use types::{
    ActiveSettingsSlider, AreaKind, CarriedItem, ConfirmDialogAction, ConfirmDialogKind,
    ConfirmDialogState, ConverterAction, GeneratorAction, HotbarItems, InventoryItems,
    LabelerAction, MainMenuAction, OpenSettingsDropdown, PauseAction, PendingAppExit,
    PendingKeyBind, SaveListAction, SettingsAction, SettingsSlider, SettingsTab, TeleportAction,
    UiPanelContext, UiRuntime, HOTBAR_SLOTS,
};

use crate::game::systems::menus::{
    confirm_dialog_actions, converter_menu_actions, generator_menu_actions, labeler_menu_actions,
    settings_menu_actions, teleport_menu_actions, teleport_rename_input,
};
use crate::game::{player, systems as game_systems};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SettingsTab::default())
            .insert_resource(OpenSettingsDropdown::default())
            .insert_resource(PendingKeyBind::default())
            .insert_resource(ActiveSettingsSlider::default())
            .insert_resource(UiRuntime::default())
            .insert_resource(ConfirmDialogState::default())
            .insert_resource(CarriedItem::default())
            .add_systems(
                Update,
                (
                    generator_menu_actions,
                    labeler_menu_actions,
                    converter_menu_actions,
                    teleport_menu_actions,
                    teleport_rename_input,
                    confirm_dialog_actions,
                    settings_menu_actions,
                )
                    .chain()
                    .after(game_systems::debug::mark_perf_input)
                    .before(game_systems::debug::mark_perf_menus),
            )
            .add_systems(
                Update,
                (
                    inventory_slot_clicks,
                    update_status_ui,
                    update_localized_ui,
                    update_button_hover_ui,
                    update_settings_text_ui,
                    update_settings_sliders_ui,
                    update_settings_slider_drag_ui,
                    update_settings_dropdowns_ui,
                    update_settings_tabs_ui,
                    update_scroll_containers,
                )
                    .after(game_systems::debug::mark_perf_animation)
                    .before(game_systems::debug::mark_perf_ui),
            )
            .add_systems(
                Update,
                (
                    update_panel_visibility,
                    update_hud_visibility,
                    update_generator_ui,
                    update_labeler_ui,
                    update_converter_ui,
                    update_teleport_ui,
                    update_inventory_slots,
                    update_carried_item_ui,
                    update_save_list_ui,
                    update_confirm_dialog_ui,
                    apply_ui_font,
                    player::controller::sync_cursor_grab,
                )
                    .after(game_systems::debug::mark_perf_animation)
                    .before(game_systems::debug::mark_perf_ui),
            );
    }
}
