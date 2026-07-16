use bevy::prelude::*;

use crate::game::session::{
    exit_to_main_menu_in_world, puzzle_save_needs_confirm, reset_solution_in_world,
    switch_to_edit_mode_in_world,
};
use crate::game::ui::access::i18n;
use crate::game::ui::core::confirm_dialog::{ConfirmExtraButton, ConfirmProps, ConfirmResult};
use crate::game::ui::features::save::open_save_puzzle_confirm_before_exit;
use crate::shared::save::SaveState;

pub const EXTRA_DISCARD: u32 = 0;

pub fn reset_solution_spec() -> ConfirmProps {
    ConfirmProps {
        title: i18n.t("confirm.title"),
        message: i18n.t("confirm.reset_solution"),
        confirm_text: i18n.t("button.confirm_reset_solution"),
        cancel_text: i18n.t("button.cancel"),
        extra: None,
    }
}

pub fn return_to_main_spec() -> ConfirmProps {
    ConfirmProps {
        title: i18n.t("confirm.title"),
        message: i18n.t("confirm.return_to_main"),
        confirm_text: i18n.t("button.save_and_back"),
        cancel_text: i18n.t("button.cancel"),
        extra: Some(ConfirmExtraButton {
            text: i18n.t("button.discard_and_back"),
            tag: EXTRA_DISCARD,
        }),
    }
}

pub fn save_before_edit_spec() -> ConfirmProps {
    ConfirmProps {
        title: i18n.t("confirm.title"),
        message: i18n.t("confirm.save_solution_before_edit"),
        confirm_text: i18n.t("button.save_solution_and_edit"),
        cancel_text: i18n.t("button.cancel"),
        extra: Some(ConfirmExtraButton {
            text: i18n.t("button.discard_solution_and_edit"),
            tag: EXTRA_DISCARD,
        }),
    }
}

pub fn on_reset_solution(result: ConfirmResult, world: &mut World) {
    if matches!(result, ConfirmResult::Confirmed) {
        reset_solution_in_world(world);
    }
}

pub fn on_return_to_main(result: ConfirmResult, world: &mut World) {
    match result {
        ConfirmResult::Confirmed => {
            if puzzle_save_needs_confirm(world.resource::<SaveState>()) {
                open_save_puzzle_confirm_before_exit();
            } else {
                exit_to_main_menu_in_world(world, true, false);
            }
        }
        ConfirmResult::Extra(EXTRA_DISCARD) => exit_to_main_menu_in_world(world, false, false),
        ConfirmResult::Cancelled | ConfirmResult::Extra(_) => {}
    }
}

pub fn on_save_before_edit(result: ConfirmResult, world: &mut World) {
    match result {
        ConfirmResult::Confirmed => switch_to_edit_mode_in_world(world, true),
        ConfirmResult::Extra(EXTRA_DISCARD) => switch_to_edit_mode_in_world(world, false),
        ConfirmResult::Cancelled | ConfirmResult::Extra(_) => {}
    }
}
