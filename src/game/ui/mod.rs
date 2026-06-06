mod components;
mod layout;
mod screens;
mod systems;
mod types;
mod widgets;

use bevy::prelude::*;

pub use crate::game::state::UiPanelId;
pub use layout::setup_ui;
pub use systems::{
    apply_ui_font, center_new_panels, inventory_slot_clicks, load_ui_font, panel_close_clicked,
    panel_drag_ended, panel_drag_started, panel_dragged, ui_hovered, ui_unhovered,
    update_block_panel_dropdowns_ui, update_carried_item_ui, update_confirm_dialog_ui,
    update_converter_ui, update_generator_ui, update_hud_visibility, update_inventory_slots,
    update_labeler_ui, update_localized_ui, update_panel_visibility, update_save_list_ui,
    update_settings_dropdowns_ui, update_settings_slider_drag_ui, update_settings_sliders_ui,
    update_settings_tabs_ui, update_settings_text_ui, update_status_ui, update_teleport_ui,
    update_text_prompt_ui, update_ui_layers,
};
pub use types::{
    ActiveSettingsSlider, AreaKind, BlockEditAction, BlockPanelDropdown, CarriedItem,
    ConfirmDialogAction, ConfirmDialogKind, ConfirmDialogState, HotbarItems, InventoryItems,
    MenuAction, OpenBlockPanelDropdown, OpenSettingsDropdown, PanelDragState, PendingKeyBind,
    SaveListAction, SaveListRenderState, SettingsAction, SettingsSliderTrigger, SettingsTab,
    TeleportAction, TextPromptAction, TextPromptKind, TextPromptState, UiHoverState,
    UiPanelContext, UiRuntime, HOTBAR_SLOTS,
};

use crate::game::systems::menus::{
    block_edit_actions, confirm_dialog_actions, settings_action_clicked, settings_menu_actions,
    teleport_menu_actions, teleport_rename_input, text_prompt_actions, text_prompt_input,
};
use crate::game::systems::debug::PerfMark;
use crate::game::player;
use components::{
    button_hovered, button_pressed, button_released, button_unhovered, update_scroll_containers,
};

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
            .insert_resource(TextPromptState::default())
            .insert_resource(SaveListRenderState::default())
            .insert_resource(CarriedItem::default())
            .insert_resource(PanelDragState::default())
            .insert_resource(UiHoverState::default())
            .add_observer(block_edit_actions)
            .add_observer(teleport_menu_actions)
            .add_observer(confirm_dialog_actions)
            .add_observer(settings_action_clicked)
            .add_observer(text_prompt_actions)
            .add_observer(inventory_slot_clicks)
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
                    teleport_rename_input,
                    text_prompt_input,
                    settings_menu_actions,
                )
                    .chain()
                    .after(PerfMark::Input)
                    .before(PerfMark::Menus),
            )
            .add_systems(
                Update,
                (
                    update_status_ui,
                    update_localized_ui,
                    update_settings_text_ui,
                    (update_settings_sliders_ui, update_settings_slider_drag_ui).chain(),
                    update_settings_dropdowns_ui,
                    update_block_panel_dropdowns_ui,
                    update_settings_tabs_ui,
                    update_scroll_containers,
                )
                    .after(PerfMark::Animation)
                    .before(PerfMark::Ui),
            )
            .add_systems(
                Update,
                (
                    (update_panel_visibility, center_new_panels, update_ui_layers).chain(),
                    update_hud_visibility,
                    update_generator_ui,
                    update_labeler_ui,
                    update_converter_ui,
                    update_teleport_ui,
                    update_inventory_slots,
                    update_carried_item_ui,
                    update_save_list_ui,
                    update_confirm_dialog_ui,
                    update_text_prompt_ui,
                    apply_ui_font,
                    player::controller::sync_cursor_grab,
                )
                    .after(PerfMark::Animation)
                    .before(PerfMark::Ui),
            );
    }
}
