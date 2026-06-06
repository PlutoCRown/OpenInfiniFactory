use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::ui::core::confirm_dialog::{ActiveConfirmDialog, ConfirmOpen, ConfirmResult};
use crate::shared::i18n::I18n;
use crate::shared::save::{delete_save, SaveState};

#[derive(SystemParam)]
pub struct SaveDialogParams<'w> {
    pub confirm: ActiveConfirmDialog<'w>,
    pub i18n: Res<'w, I18n>,
}

pub fn open_delete_confirm(params: &mut SaveDialogParams, name: String) {
    let spec = ConfirmOpen {
        title: params.i18n.text("confirm.title"),
        message: params.i18n.fmt("save.confirm_delete", &[("name", name.clone())]),
        confirm_text: params.i18n.text("button.delete"),
        cancel_text: params.i18n.text("button.cancel"),
        extra: None,
    };
    params.confirm.open_then(spec, move |result, world| {
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
