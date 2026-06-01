use bevy::prelude::*;

use crate::game::state::GameMode;
use crate::game::ui::components::{spawn_panel, PanelOptions};
use crate::game::ui::types::{ButtonSpec, PanelVisibility, PauseMenuAction};
use crate::shared::i18n::I18n;

mod actions;
mod widgets;

pub(crate) use actions::pause_menu_actions;

use widgets::spawn_menu_button;

const PAUSE_MENU_ITEMS: &[ButtonSpec<PauseMenuAction>] = &[
    ButtonSpec::new("button.resume", PauseMenuAction::Resume),
    ButtonSpec::new(
        "button.toggle_builder_mode",
        PauseMenuAction::ToggleBuilderMode,
    ),
    ButtonSpec::new("button.save_world", PauseMenuAction::SaveWorld),
    ButtonSpec::new(
        "button.save_as_new_puzzle",
        PauseMenuAction::SaveAsNewPuzzle,
    ),
    ButtonSpec::new("button.reset_solution", PauseMenuAction::ResetSolution),
    ButtonSpec::new("button.settings", PauseMenuAction::OpenSettings),
    ButtonSpec::new("button.back_to_main_menu", PauseMenuAction::BackToMainMenu),
];

pub fn spawn_pause_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(420.0, "state.paused").title_size(30.0),
        PanelVisibility::GameMode(GameMode::Paused),
        |panel| {
            for item in PAUSE_MENU_ITEMS {
                spawn_menu_button(panel, 38.0, 16.0, item.on_click, item.text);
            }
        },
    );
}
