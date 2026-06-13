use bevy::prelude::*;

use crate::game::session::{exit_to_main_menu_in_world, save_current_world_invalidate_in_world};
use crate::game::ui::access::{i18n, ui};
use crate::game::ui::core::confirm_dialog::{ConfirmExtraButton, ConfirmProps, ConfirmResult};
use crate::shared::save::{
    delete_save, invalidate_solutions_for_puzzle, save_kind, SaveKind, SaveState,
};

use super::prompt::open_save_as_new_puzzle_prompt;

pub const EXTRA_SAVE_AS: u32 = 1;

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
        if save_kind(&name) == Some(SaveKind::Puzzle) {
            invalidate_solutions_for_puzzle(&name);
        }
        delete_save(&name);
        let mut save_state = world.resource_mut::<SaveState>();
        save_state.refresh();
        if save_state.selected_puzzle.as_deref() == Some(name.as_str()) {
            save_state.select_puzzle(None);
        }
    });
}

pub fn open_save_puzzle_confirm() {
    let spec = ConfirmProps {
        title: i18n.t("confirm.title"),
        message: i18n.t("confirm.save_puzzle_invalidate_solutions"),
        confirm_text: i18n.t("button.save_anyway"),
        cancel_text: i18n.t("button.cancel"),
        extra: Some(ConfirmExtraButton {
            text: i18n.t("button.save_as"),
            tag: EXTRA_SAVE_AS,
        }),
    };
    ui.open_confirm_then(spec, on_save_puzzle_confirm);
}

pub fn on_save_puzzle_confirm(result: ConfirmResult, world: &mut World) {
    match result {
        ConfirmResult::Confirmed => save_current_world_invalidate_in_world(world),
        ConfirmResult::Extra(EXTRA_SAVE_AS) => open_save_as_new_puzzle_prompt(),
        ConfirmResult::Cancelled | ConfirmResult::Extra(_) => {}
    }
}

pub fn open_save_puzzle_confirm_before_exit() {
    let spec = ConfirmProps {
        title: i18n.t("confirm.title"),
        message: i18n.t("confirm.save_puzzle_invalidate_solutions"),
        confirm_text: i18n.t("button.save_anyway"),
        cancel_text: i18n.t("button.cancel"),
        extra: Some(ConfirmExtraButton {
            text: i18n.t("button.save_as"),
            tag: EXTRA_SAVE_AS,
        }),
    };
    ui.open_confirm_then(spec, on_save_puzzle_confirm_before_exit);
}

pub fn on_save_puzzle_confirm_before_exit(result: ConfirmResult, world: &mut World) {
    match result {
        ConfirmResult::Confirmed => {
            save_current_world_invalidate_in_world(world);
            exit_to_main_menu_in_world(world, false);
        }
        ConfirmResult::Extra(EXTRA_SAVE_AS) => open_save_as_new_puzzle_prompt(),
        ConfirmResult::Cancelled | ConfirmResult::Extra(_) => {}
    }
}
