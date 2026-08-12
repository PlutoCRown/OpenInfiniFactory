use bevy::prelude::*;

use crate::game::ui::core::StartMenuPage;
use crate::game::ui::features::pause_menu::spawn_pause_menu_buttons;
use crate::game::ui::features::start_menu::spawn_start_menu_buttons;

use super::super::components::{PanelOptions, spawn_panel};
use super::super::types::PanelVisibility;

pub fn spawn_main_menu(root: &mut ChildSpawnerCommands) {
    spawn_panel(
        root,
        PanelOptions::new(420.0, "main.title"),
        PanelVisibility::StartMenuPage(StartMenuPage::Main),
        spawn_start_menu_buttons,
    );
}

pub fn spawn_pause_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel(
        root,
        PanelOptions::new(420.0, "state.paused"),
        PanelVisibility::PauseMenu,
        spawn_pause_menu_buttons,
    );
}
