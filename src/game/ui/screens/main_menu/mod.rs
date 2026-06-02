use bevy::prelude::*;

use crate::game::state::GameMode;
use crate::game::ui::components::{spawn_panel, PanelOptions};
use crate::game::ui::types::{
    ButtonSpec, MainMenuAction, PanelVisibility, UiPanelBinding, UiPanelKey,
};
use crate::shared::i18n::I18n;

mod actions;
mod widgets;

pub(crate) use actions::main_menu_actions;

use widgets::spawn_menu_button;

const MAIN_MENU_ITEMS: &[ButtonSpec<MainMenuAction>] = &[
    ButtonSpec::new("button.edit_puzzle", MainMenuAction::EditPuzzle),
    ButtonSpec::new("button.start_playing", MainMenuAction::Play),
    ButtonSpec::new("button.settings", MainMenuAction::OpenSettings),
    ButtonSpec::new("button.quit_game", MainMenuAction::Quit),
];

pub fn spawn_main_menu(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(420.0, "main.title").title_size(30.0),
        (
            PanelVisibility::GameMode(GameMode::MainMenu),
            UiPanelBinding(UiPanelKey::MAIN_MENU),
        ),
        |panel| {
            for item in MAIN_MENU_ITEMS {
                spawn_menu_button(panel, 44.0, 17.0, item.on_click, item.text);
            }
        },
    )
}
