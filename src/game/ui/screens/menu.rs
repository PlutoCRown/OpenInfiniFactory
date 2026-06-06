use bevy::prelude::*;

use crate::game::state::StartMenuScreen;

use super::super::components::{spawn_panel, PanelOptions};
use super::super::types::{MenuAction, PanelVisibility};
use super::super::widgets::spawn_menu_button;

#[derive(Clone, Copy)]
struct MenuItem {
    action: MenuAction,
    height: f32,
    font_size: f32,
}

const MAIN_MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        action: MenuAction::EditPuzzle,
        height: 44.0,
        font_size: 17.0,
    },
    MenuItem {
        action: MenuAction::Play,
        height: 44.0,
        font_size: 17.0,
    },
    MenuItem {
        action: MenuAction::OpenSettings,
        height: 44.0,
        font_size: 17.0,
    },
    MenuItem {
        action: MenuAction::Quit,
        height: 44.0,
        font_size: 17.0,
    },
];

const PAUSE_MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        action: MenuAction::Resume,
        height: 38.0,
        font_size: 16.0,
    },
    MenuItem {
        action: MenuAction::ToggleBuilderMode,
        height: 38.0,
        font_size: 16.0,
    },
    MenuItem {
        action: MenuAction::SaveWorld,
        height: 38.0,
        font_size: 16.0,
    },
    MenuItem {
        action: MenuAction::SaveAsNewPuzzle,
        height: 38.0,
        font_size: 16.0,
    },
    MenuItem {
        action: MenuAction::ResetSolution,
        height: 38.0,
        font_size: 16.0,
    },
    MenuItem {
        action: MenuAction::OpenSettings,
        height: 38.0,
        font_size: 16.0,
    },
    MenuItem {
        action: MenuAction::BackToMainMenu,
        height: 38.0,
        font_size: 16.0,
    },
];

pub fn spawn_main_menu(root: &mut ChildSpawnerCommands) {
    spawn_menu_panel(
        root,
        PanelOptions::new(420.0, "main.title").title_size(30.0),
        PanelVisibility::StartMenuScreen(StartMenuScreen::Main),
        MAIN_MENU_ITEMS,
    );
}

pub fn spawn_pause_panel(root: &mut ChildSpawnerCommands) {
    spawn_menu_panel(
        root,
        PanelOptions::new(420.0, "state.paused").title_size(30.0),
        PanelVisibility::PauseMenu,
        PAUSE_MENU_ITEMS,
    );
}

fn spawn_menu_panel(
    root: &mut ChildSpawnerCommands,
    options: PanelOptions,
    visibility: PanelVisibility,
    items: &[MenuItem],
) {
    spawn_panel(root, options, visibility, |panel| {
        for item in items {
            spawn_menu_button(panel, item.height, item.font_size, item.action);
        }
    });
}
