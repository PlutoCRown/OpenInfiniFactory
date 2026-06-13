use bevy::prelude::*;

use crate::game::session;
use crate::game::ui::access::{i18n, ui};
use crate::game::ui::core::text_prompt::{TextPromptProps, TextPromptResult};
use crate::shared::save::{next_named_save, rename_save, SaveState};

pub fn text_prompt_spec(title_key: &'static str, default_value: &str) -> TextPromptProps {
    TextPromptProps {
        title: i18n.t(title_key),
        default_value: default_value.to_string(),
        save_text: i18n.t("button.confirm"),
        cancel_text: i18n.t("button.cancel"),
    }
}

fn resolved_name(
    save_state: &SaveState,
    requested: &str,
    rename_from: Option<&str>,
) -> Option<String> {
    if let Some(old) = rename_from {
        if requested.trim() == old {
            return Some(old.to_string());
        }
    }
    let existing = save_state
        .entries
        .iter()
        .map(|entry| entry.name.clone())
        .collect::<Vec<_>>();
    let name = next_named_save(&existing, requested);
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

pub fn open_new_puzzle_prompt() {
    let spec = text_prompt_spec("save.prompt.new_puzzle", "puzzle");
    ui.open_text_prompt_then(spec, |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        let save_state = world.resource::<SaveState>();
        let Some(name) = resolved_name(save_state, &requested, None) else {
            return;
        };
        session::create_new_puzzle_in_world(world, name);
    });
}

pub fn open_new_solution_prompt(puzzle: String) {
    let spec = text_prompt_spec("save.prompt.new_solution", "solution");
    ui.open_text_prompt_then(spec, move |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        let save_state = world.resource::<SaveState>();
        let Some(name) = resolved_name(save_state, &requested, None) else {
            return;
        };
        session::create_new_solution_in_world(world, name, puzzle);
    });
}

pub fn open_rename_puzzle_prompt(old_name: String) {
    let spec = text_prompt_spec("save.prompt.rename_puzzle", old_name.as_str());
    ui.open_text_prompt_then(spec, move |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        let mut save_state = world.resource_mut::<SaveState>();
        let Some(name) = resolved_name(&save_state, &requested, Some(old_name.as_str())) else {
            return;
        };
        if old_name == name || rename_save(&old_name, &name) {
            if save_state.current.as_deref() == Some(old_name.as_str()) {
                save_state.current = Some(name.clone());
            }
            if save_state.selected_puzzle.as_deref() == Some(old_name.as_str()) {
                save_state.select_puzzle(Some(name.clone()));
            }
            save_state.refresh();
        }
    });
}

pub fn open_rename_solution_prompt(old_name: String) {
    let spec = text_prompt_spec("save.prompt.rename_solution", old_name.as_str());
    ui.open_text_prompt_then(spec, move |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        let mut save_state = world.resource_mut::<SaveState>();
        let Some(name) = resolved_name(&save_state, &requested, Some(old_name.as_str())) else {
            return;
        };
        if old_name == name || rename_save(&old_name, &name) {
            if save_state.current.as_deref() == Some(old_name.as_str()) {
                save_state.current = Some(name.clone());
            }
            if save_state.selected_puzzle.as_deref() == Some(old_name.as_str()) {
                save_state.select_puzzle(Some(name.clone()));
            }
            save_state.refresh();
        }
    });
}

pub fn open_save_as_new_puzzle_prompt() {
    let spec = text_prompt_spec("save.prompt.save_as_new_puzzle", "puzzle");
    ui.open_text_prompt_then(spec, |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        let save_state = world.resource::<SaveState>();
        let Some(name) = resolved_name(save_state, &requested, None) else {
            return;
        };
        session::save_world_as_new_puzzle_in_world(world, name);
    });
}
