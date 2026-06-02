pub(crate) mod components;
mod demo_panel;
mod layout;
mod screens;
mod systems;
pub(crate) mod types;

use bevy::prelude::*;

pub use crate::game::state::UiPanelId;
pub use layout::setup_ui;
pub(crate) use screens::{
    inventory_slot_clicks, main_menu_actions, pause_menu_actions, save_list_actions,
    update_carried_item_ui, update_confirm_dialog_ui, update_inventory_slots, update_save_list_ui,
    update_settings_dropdowns_ui, update_settings_slider_drag_ui, update_settings_sliders_ui,
    update_settings_tabs_ui, update_settings_text_ui, update_text_prompt_ui,
};
pub use systems::{
    apply_ui_font, center_new_panels, cleanup_closed_panel_state, load_ui_font, open_initial_panel,
    panel_close_clicked, panel_drag_ended, panel_drag_started, panel_dragged,
    register_legacy_panels, sync_mode_panels, ui_hovered, ui_unhovered, update_hud_visibility,
    update_localized_ui, update_panel_visibility, update_status_ui, update_ui_layers,
};
pub use types::{
    ActiveSettingsSlider, AreaKind, BlockEditAction, BlockPanelDropdown, BlockSettingsChanged,
    CarriedItem, CloseUiModal, CloseUiPanel, ConfirmDialogAction, ConfirmDialogButtonSpec,
    ConfirmDialogEffect, ConfirmDialogMessage, ConfirmDialogResult, ConfirmDialogSpec,
    GameplayUiChanged, HotbarItems, InventoryChanged, InventoryItems, LanguageChanged,
    MainMenuAction, OpenBlockPanelDropdown, OpenConfirmDialog, OpenSettingsDropdown,
    OpenTextPrompt, OpenUiPanel, PanelDragState, PauseMenuAction, PendingKeyBind,
    SaveListAction, SaveListChanged, SaveListRenderState, SettingsAction, SettingsChanged,
    SettingsSliderTrigger, SettingsTab, TeleportAction, TextPromptAction, TextPromptKind,
    UiHoverState, UiModalClosed, UiModalKind, UiModalOpened, UiPanelBinding, UiPanelClosed,
    UiPanelContext, UiPanelContextChanged, UiPanelDescriptor, UiPanelHost, UiPanelKey,
    UiPanelOpened, UiPanelRegistry, UiRuntime, HOTBAR_SLOTS,
};

pub(crate) use crate::game::ui::demo_panel::{open_demo_panel_shortcut, register_demo_panel};
use crate::game::ui::screens::{
    cleanup_closed_settings_panel, confirm_dialog_actions, settings_action_clicked,
    settings_menu_actions, text_prompt_actions, text_prompt_input,
};
use crate::game::world::blocks::{
    block_edit_actions, teleport_menu_actions, teleport_rename_input,
    update_block_panel_dropdowns_ui, update_converter_ui, update_generator_ui, update_labeler_ui,
    update_teleport_ui,
};
use crate::game::{player, systems as game_systems};
use components::{update_button_interactions, update_scroll_containers};
use systems::{
    close_panel_messages, modal_messages, open_panel_button_clicked, open_panel_messages,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SettingsTab::default())
            .init_resource::<UiPanelRegistry>()
            .init_resource::<UiPanelHost>()
            .add_message::<OpenUiPanel>()
            .add_message::<CloseUiPanel>()
            .add_message::<OpenConfirmDialog>()
            .add_message::<OpenTextPrompt>()
            .add_message::<CloseUiModal>()
            .add_message::<UiPanelOpened>()
            .add_message::<UiPanelClosed>()
            .add_message::<UiPanelContextChanged>()
            .add_message::<UiModalOpened>()
            .add_message::<UiModalClosed>()
            .add_message::<SettingsChanged>()
            .add_message::<InventoryChanged>()
            .add_message::<LanguageChanged>()
            .add_message::<SaveListChanged>()
            .add_message::<GameplayUiChanged>()
            .add_message::<BlockSettingsChanged>()
            .insert_resource(OpenSettingsDropdown::default())
            .insert_resource(OpenBlockPanelDropdown::default())
            .insert_resource(PendingKeyBind::default())
            .insert_resource(ActiveSettingsSlider::default())
            .insert_resource(UiRuntime::default())
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
            .add_observer(open_panel_button_clicked)
            .add_observer(panel_close_clicked)
            .add_observer(panel_drag_started)
            .add_observer(panel_dragged)
            .add_observer(panel_drag_ended)
            .add_observer(ui_hovered)
            .add_observer(ui_unhovered)
            .add_systems(Startup, (register_legacy_panels, register_demo_panel))
            .add_systems(PostStartup, open_initial_panel)
            .add_systems(
                Update,
                (
                    open_demo_panel_shortcut,
                    sync_mode_panels,
                    main_menu_actions,
                    open_panel_messages,
                    close_panel_messages,
                    modal_messages,
                    cleanup_closed_panel_state,
                )
                    .chain()
                    .after(game_systems::debug::mark_perf_input)
                    .before(game_systems::debug::mark_perf_menus),
            )
            .add_systems(
                Update,
                (
                    teleport_rename_input,
                    text_prompt_input,
                    settings_menu_actions,
                    cleanup_closed_settings_panel,
                )
                    .chain()
                    .after(game_systems::debug::mark_perf_input)
                    .before(game_systems::debug::mark_perf_menus),
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
                    update_button_interactions,
                    update_scroll_containers,
                )
                    .after(game_systems::debug::mark_perf_animation)
                    .before(game_systems::debug::mark_perf_ui),
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
                    .after(game_systems::debug::mark_perf_animation)
                    .before(game_systems::debug::mark_perf_ui),
            );
    }
}
