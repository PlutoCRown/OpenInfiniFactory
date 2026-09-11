use bevy::prelude::*;

use crate::game::input::GameplayInputState;
use crate::game::state::{GameMode, SolutionState, WorldEntryMode};
use crate::game::ui::PanelCloseDeps;
use crate::game::ui::access::ui;
use crate::game::ui::core::host::UiRootEntity;
use crate::game::ui::core::runtime::UiPanelContext;
use crate::game::ui::core::{StartMenuPage, UiNavigation};
use crate::game::ui::menu_button::{MenuButtonClick, MenuButtonSet, spawn_menu_button};
use crate::shared::save::SaveState;

use super::start_menu_mounts::sync_start_menu_mounts;

pub struct StartMenuPlugin;

/// 主菜单按钮只绑定业务系统，不持有业务资源集合。
struct StartMenuButton {
    label_key: &'static str,
    on_click: fn(&mut Commands),
}

const START_MENU_BUTTONS: &[StartMenuButton] = &[
    StartMenuButton {
        label_key: "button.start_playing",
        on_click: |commands| {
            commands.run_system_cached(open_save_list);
        },
    },
    StartMenuButton {
        label_key: "button.settings",
        on_click: |commands| {
            commands.run_system_cached(open_start_menu_settings);
        },
    },
    #[cfg(not(target_arch = "wasm32"))]
    StartMenuButton {
        label_key: "button.quit_game",
        on_click: |_commands| std::process::exit(0),
    },
];

impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                dispatch_start_menu_clicks.in_set(crate::game::schedule::GameSet::Menus),
                start_menu_escape.in_set(crate::game::schedule::GameSet::Menus),
                sync_start_menu_mounts.in_set(crate::game::schedule::GameSet::Menus),
            ),
        );
    }
}

pub fn spawn_start_menu_buttons(panel: &mut ChildSpawnerCommands) {
    for (index, button) in START_MENU_BUTTONS.iter().enumerate() {
        spawn_menu_button(
            panel,
            MenuButtonSet::StartMenu,
            index as u8,
            button.label_key,
        );
    }
}

fn dispatch_start_menu_clicks(
    mut clicks: MessageReader<MenuButtonClick>,
    mode: Res<State<GameMode>>,
    mut commands: Commands,
) {
    if *mode.get() != GameMode::StartMenu {
        return;
    }
    for click in clicks.read() {
        if click.set != MenuButtonSet::StartMenu {
            continue;
        }
        let Some(button) = START_MENU_BUTTONS.get(click.index as usize) else {
            continue;
        };
        (button.on_click)(&mut commands);
    }
}

/// 开始游玩业务系统刷新存档并进入存档选择页。
fn open_save_list(
    mut navigation: ResMut<UiNavigation>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
) {
    save_state.refresh();
    save_state.select_puzzle(None);
    save_state.select_solution(None);
    solution_state.save_list_entry = WorldEntryMode::PlaySolution;
    navigation.show_start_menu(StartMenuPage::SaveList);
}

/// 设置按钮系统只声明挂载设置面板所需的根实体。
fn open_start_menu_settings(ui_root: Option<Res<UiRootEntity>>, mut commands: Commands) {
    ui.mount_settings(
        &mut commands,
        ui_root.as_deref().map(|root| root.0),
        UiPanelContext::SettingsFromStartMenu,
    );
}

/// 主菜单 Esc：关设置/确认/输入，或从存档列表返回
fn start_menu_escape(
    input: Res<GameplayInputState>,
    mode: Res<State<GameMode>>,
    mut close: PanelCloseDeps,
    mut commands: Commands,
) {
    if *mode.get() != GameMode::StartMenu || !input.pause {
        return;
    }
    // 改键 / 输入中：交给各自逻辑，不在这里抢 Esc
    if close.pending_key_bind.0.is_some()
        || close.text_prompt.is_open()
        || close.inline_edit.is_active()
    {
        return;
    }
    let _ = close.dismiss_start_menu_overlay(&mut commands);
}
