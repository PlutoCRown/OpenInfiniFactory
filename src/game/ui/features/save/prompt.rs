use bevy::prelude::*;

use crate::game::session;
use crate::game::ui::access::{i18n, ui};
use crate::game::ui::core::text_prompt::{TextPromptProps, TextPromptResult};
use crate::shared::save::{rename_save_to, SaveSlot, SaveState};

pub fn text_prompt_spec(title_key: &'static str, default_value: &str) -> TextPromptProps {
    TextPromptProps {
        title: i18n.t(title_key),
        default_value: default_value.to_string(),
        save_text: i18n.t("button.confirm"),
        cancel_text: i18n.t("button.cancel"),
    }
}

pub fn open_new_puzzle_prompt() {
    let spec = text_prompt_spec("save.prompt.new_puzzle", "puzzle");
    ui.open_text_prompt_then(spec, |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        let name = requested.trim().to_string();
        if name.is_empty() {
            return;
        }
        session::create_new_puzzle_in_world(world, name);
    });
}

pub fn open_new_solution_prompt(puzzle: String) {
    let spec = text_prompt_spec("save.prompt.new_solution", "solution");
    ui.open_text_prompt_then(spec, move |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        let name = requested.trim().to_string();
        if name.is_empty() {
            return;
        }
        session::create_new_solution_in_world(world, name, puzzle);
    });
}

pub fn open_rename_puzzle_prompt(slot: SaveSlot, name: String) {
    let spec = text_prompt_spec("save.prompt.rename_puzzle", name.as_str());
    ui.open_text_prompt_then(spec, move |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        if requested.trim() == name {
            return;
        }
        let mut save_state = world.resource_mut::<SaveState>();
        let Some(new_slot) = rename_save_to(&slot, &requested) else {
            return;
        };
        if save_state.current.as_ref() == Some(&slot) {
            save_state.current = Some(new_slot.clone());
        }
        if save_state.selected_puzzle.as_deref() == Some(slot.puzzle.as_str()) {
            save_state.select_puzzle(Some(new_slot.puzzle.clone()));
        }
        save_state.refresh();
    });
}

pub fn open_rename_solution_prompt(slot: SaveSlot, name: String) {
    let spec = text_prompt_spec("save.prompt.rename_solution", name.as_str());
    ui.open_text_prompt_then(spec, move |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        if requested.trim() == name {
            return;
        }
        let mut save_state = world.resource_mut::<SaveState>();
        let Some(new_slot) = rename_save_to(&slot, &requested) else {
            return;
        };
        if save_state.current.as_ref() == Some(&slot) {
            save_state.current = Some(new_slot);
        }
        save_state.refresh();
    });
}

pub fn open_save_as_new_puzzle_prompt() {
    let spec = text_prompt_spec("save.prompt.save_as_new_puzzle", "puzzle");
    ui.open_text_prompt_then(spec, |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        let name = requested.trim().to_string();
        if name.is_empty() {
            return;
        }
        session::save_world_as_new_puzzle_in_world(world, name);
    });
}
