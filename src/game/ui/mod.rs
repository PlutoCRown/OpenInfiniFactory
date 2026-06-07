pub mod access;
pub(crate) mod components;
pub mod core;
pub mod features;
mod layout;
mod screens;
mod systems;
pub(crate) mod types;
mod widgets;

use bevy::prelude::*;

#[allow(unused_imports)]
pub use access::{bind_ui_scope, i18n, ui, I18nRevision, UiAccessScope};
pub use layout::{setup_menu_ui, setup_playing_ui_system};
pub use systems::{
    apply_ui_font, dismiss_playing_overlay, load_ui_font, panel_close_clicked, panel_drag_ended,
    panel_drag_started, panel_dragged, ui_hovered, ui_unhovered, update_hud_visibility,
    update_localized_ui, update_panel_visibility, update_status_ui, update_ui_layers,
};
pub use types::*;

use crate::game::systems::perf::PerfScope;
use crate::game::ui::core::confirm_dialog::{
    emit_confirm_dialog_actions, update_confirm_dialog_ui, PendingConfirmHandler,
};
use crate::game::ui::core::host::UiAction;
use crate::game::ui::core::text_prompt::{
    emit_text_prompt_actions, update_text_prompt_ui, PendingTextPromptHandler,
};
use access::unbind_ui_scope;
use components::{
    button_hovered, button_pressed, button_released, button_unhovered, update_scroll_containers,
};
use features::UiFeaturesPlugin;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, UiAccessScope)
            .add_systems(Update, bind_ui_scope.before(UiAccessScope))
            .add_systems(Update, unbind_ui_scope.after(UiAccessScope))
            .add_message::<UiAction>()
            .insert_resource(UiRuntime::default())
            .insert_resource(I18nRevision::default())
            .insert_resource(crate::game::ui::core::host::UiHost::default())
            .insert_resource(crate::game::ui::core::text_prompt::TextPromptState::default())
            .insert_resource(crate::game::ui::core::confirm_dialog::ConfirmDialogState::default())
            .insert_non_send_resource(PendingConfirmHandler::default())
            .insert_non_send_resource(PendingTextPromptHandler::default())
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
            .add_observer(emit_confirm_dialog_actions)
            .add_observer(emit_text_prompt_actions)
            .add_systems(
                Update,
                (
                    update_localized_ui,
                    update_text_prompt_ui,
                    update_confirm_dialog_ui,
                    update_scroll_containers,
                    apply_ui_font,
                )
                    .in_set(UiAccessScope)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            )
            .add_systems(
                Update,
                (update_panel_visibility, update_ui_layers)
                    .chain()
                    .in_set(UiAccessScope)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            )
            .add_systems(
                Update,
                (update_status_ui, update_hud_visibility)
                    .in_set(UiAccessScope)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            );
    }
}
