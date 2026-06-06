use bevy::prelude::*;

use crate::game::ui::access::{i18n, ui};
use crate::game::ui::core::confirm_dialog::{ConfirmProps, ConfirmResult};
use crate::shared::save::{delete_save, SaveState};

pub fn open_delete_confirm(name: String) {
    let spec = ConfirmProps {
        title: i18n.t("confirm.title"),
        message: i18n.fmt("save.confirm_delete", &[("name", name.clone())]),
        confirm_text: i18n.t("button.delete"),
        cancel_text: i18n.t("button.cancel"),
        extra: None,
    };
    ui.open_confirm_then(spec, move |result, world| {
        if !matches!(result, ConfirmResult::Confirmed) {
            return;
        }
        delete_save(&name);
        let mut save_state = world.resource_mut::<SaveState>();
        save_state.refresh();
        if save_state.selected_puzzle.as_deref() == Some(name.as_str()) {
            save_state.select_puzzle(None, None);
        }
    });
}
