use bevy::prelude::*;

use crate::game::ui::core::StartMenuPage;
use crate::game::ui::features::pause_menu::spawn_pause_menu_buttons;
use crate::game::ui::features::start_menu::spawn_start_menu_buttons;

use super::super::components::{PanelOptions, spawn_panel};
use super::super::types::PanelVisibility;

/// 主菜单在宽屏占左侧区域，窄屏自动退回全宽布局。
pub fn spawn_main_menu(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(640.0),
            height: Val::Percent(100.0),
            padding: UiRect::horizontal(Val::Percent(8.0)),
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        Pickable::IGNORE,
    ))
    .with_children(|menu| {
        spawn_panel(
            menu,
            PanelOptions {
                flow: true,
                ..PanelOptions::new(420.0, "main.title")
            },
            PanelVisibility::StartMenuPage(StartMenuPage::Main),
            spawn_start_menu_buttons,
        );
    });
}

pub fn spawn_pause_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel(
        root,
        PanelOptions::new(420.0, "state.paused"),
        PanelVisibility::PauseMenu,
        spawn_pause_menu_buttons,
    );
}
