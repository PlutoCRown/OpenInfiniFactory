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
    apply_ui_font, inventory_slot_clicks, load_ui_font, update_block_panel_dropdowns_ui,
    update_button_hover_ui, update_carried_item_ui, update_confirm_dialog_ui, update_converter_ui,
    update_generator_ui, update_hud_visibility, update_inventory_slots, update_labeler_ui,
    update_localized_ui, update_panel_visibility, update_save_list_ui, update_scroll_containers,
    update_settings_dropdowns_ui, update_settings_slider_drag_ui, update_settings_sliders_ui,
    update_settings_tabs_ui, update_settings_text_ui, update_status_ui, update_teleport_ui,
    update_ui_layers,
};
pub use types::{
    ActiveSettingsSlider, AreaKind, BlockEditAction, BlockPanelDropdown, CarriedItem,
    ConfirmDialogAction, ConfirmDialogKind, ConfirmDialogState,
    HotbarItems, InventoryItems, MenuAction, OpenBlockPanelDropdown,
    OpenSettingsDropdown, PendingAppExit, PendingKeyBind, SaveListAction,
    SettingsAction, SettingsSliderTrigger, SettingsTab, TeleportAction, UiPanelContext, UiRuntime,
    HOTBAR_SLOTS,
};

use crate::game::systems::menus::{
    block_edit_actions, confirm_dialog_actions, settings_menu_actions, teleport_menu_actions,
    teleport_rename_input,
};
use crate::game::{player, systems as game_systems};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SettingsTab::default())
            .insert_resource(OpenSettingsDropdown::default())
            .insert_resource(OpenBlockPanelDropdown::default())
            .insert_resource(PendingKeyBind::default())
            .insert_resource(ActiveSettingsSlider::default())
            .insert_resource(UiRuntime::default())
            .insert_resource(ConfirmDialogState::default())
            .insert_resource(CarriedItem::default())
            .add_systems(
                Update,
                (
                    block_edit_actions,
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
                    (update_settings_sliders_ui, update_settings_slider_drag_ui).chain(),
                    update_settings_dropdowns_ui,
                    update_block_panel_dropdowns_ui,
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
                    update_ui_layers,
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
