use bevy::prelude::*;

use crate::game::state::GameMode;
use crate::shared::i18n::I18n;

use crate::game::ui::components::{spawn_panel, PanelOptions};
use crate::game::ui::types::{ButtonSpec, MenuAction, PanelVisibility};

mod widgets;

use widgets::spawn_menu_button;

const MAIN_MENU_ITEMS: &[ButtonSpec<MenuAction>] = &[
    ButtonSpec::new("button.edit_puzzle", MenuAction::EditPuzzle),
    ButtonSpec::new("button.start_playing", MenuAction::Play),
    ButtonSpec::new("button.settings", MenuAction::OpenSettings),
    ButtonSpec::new("button.quit_game", MenuAction::Quit),
];

const PAUSE_MENU_ITEMS: &[ButtonSpec<MenuAction>] = &[
    ButtonSpec::new("button.resume", MenuAction::Resume),
    ButtonSpec::new("button.toggle_builder_mode", MenuAction::ToggleBuilderMode),
    ButtonSpec::new("button.save_world", MenuAction::SaveWorld),
    ButtonSpec::new("button.save_as_new_puzzle", MenuAction::SaveAsNewPuzzle),
    ButtonSpec::new("button.reset_solution", MenuAction::ResetSolution),
    ButtonSpec::new("button.settings", MenuAction::OpenSettings),
    ButtonSpec::new("button.back_to_main_menu", MenuAction::BackToMainMenu),
];

#[derive(Clone, Copy)]
struct MenuButtonStyle {
    height: f32,
    font_size: f32,
}

pub fn spawn_main_menu(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_menu_panel(
        root,
        i18n,
        PanelOptions::new(420.0, "main.title").title_size(30.0),
        PanelVisibility::GameMode(GameMode::MainMenu),
        MAIN_MENU_ITEMS,
        MenuButtonStyle {
            height: 44.0,
            font_size: 17.0,
        },
    );
}

pub fn spawn_pause_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_menu_panel(
        root,
        i18n,
        PanelOptions::new(420.0, "state.paused").title_size(30.0),
        PanelVisibility::GameMode(GameMode::Paused),
        PAUSE_MENU_ITEMS,
        MenuButtonStyle {
            height: 38.0,
            font_size: 16.0,
        },
    );
}

fn spawn_menu_panel(
    root: &mut ChildSpawnerCommands,
    i18n: &I18n,
    options: PanelOptions,
    visibility: PanelVisibility,
    items: &[ButtonSpec<MenuAction>],
    button_style: MenuButtonStyle,
) {
    spawn_panel(root, i18n, options, visibility, |panel| {
        for item in items {
            spawn_menu_button(
                panel,
                button_style.height,
                button_style.font_size,
                item.on_click,
                item.text,
            );
        }
    });
}
