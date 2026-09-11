mod confirm;

use bevy::prelude::*;

use crate::game::session;
use crate::game::session::{SessionBusy, puzzle_save_needs_confirm};
use crate::game::state::{BuilderMode, GameMode, SolutionState, WorldEntryMode};
use crate::game::ui::access::{UiContext, i18n, ui};
use crate::game::ui::core::host::PlayingUiRootEntity;
use crate::game::ui::core::runtime::{UiNavigation, UiPanelContext};
use crate::game::ui::features::save::{
    open_export_as_puzzle_prompt, open_save_as_new_puzzle_prompt, open_save_puzzle_confirm,
};
use crate::game::ui::menu_button::{
    MenuButtonClick, MenuButtonMarker, MenuButtonSet, spawn_menu_button,
};
use crate::shared::save::{SaveKind, SaveState};

use super::playing_overlays::sync_playing_overlay_mounts;

use confirm::{
    on_reset_solution, on_return_to_main, on_save_before_edit, reset_solution_spec,
    return_to_main_spec, save_before_edit_spec,
};

pub struct PauseMenuPlugin;

/// 按钮只声明展示规则和命令绑定，执行系统独立声明业务依赖。
struct PauseMenuButton {
    label_key: &'static str,
    label: Option<fn(&SaveState) -> String>,
    visible: Option<fn(&SaveState, &SolutionState) -> bool>,
    on_click: fn(&mut Commands),
}

/// 暂停菜单的数据化按钮表；新增业务依赖只修改所绑定的系统。
const PAUSE_MENU_BUTTONS: &[PauseMenuButton] = &[
    PauseMenuButton {
        label_key: "button.resume",
        label: None,
        visible: None,
        on_click: |commands| {
            commands.run_system_cached(resume_playing);
        },
    },
    PauseMenuButton {
        label_key: "button.toggle_builder_mode",
        label: None,
        visible: Some(|_, solution| solution.entry == WorldEntryMode::EditPuzzle),
        on_click: |commands| {
            commands.run_system_cached(toggle_builder_mode);
        },
    },
    PauseMenuButton {
        label_key: "button.save_world",
        visible: None,
        label: Some(|save| {
            i18n.t(match save.current_kind {
                Some(SaveKind::Solution) => "button.save_solution",
                Some(SaveKind::Free) => "button.save_world",
                _ => "button.save_puzzle",
            })
        }),
        on_click: |commands| {
            commands.run_system_cached(save_world);
        },
    },
    PauseMenuButton {
        label_key: "button.save_as_new_puzzle",
        label: None,
        visible: Some(|save, _| save.current_kind == Some(SaveKind::Puzzle)),
        on_click: |commands| {
            commands.run_system_cached(save_as_puzzle);
        },
    },
    PauseMenuButton {
        label_key: "button.export_as_puzzle",
        label: None,
        visible: Some(|save, _| save.current_kind == Some(SaveKind::Free)),
        on_click: |commands| {
            commands.run_system_cached(export_puzzle);
        },
    },
    PauseMenuButton {
        label_key: "button.reset_solution",
        label: None,
        visible: Some(|save, _| save.current_kind == Some(SaveKind::Solution)),
        on_click: |commands| {
            commands.run_system_cached(confirm_reset_solution);
        },
    },
    PauseMenuButton {
        label_key: "button.settings",
        label: None,
        visible: None,
        on_click: |commands| {
            commands.run_system_cached(open_settings);
        },
    },
    PauseMenuButton {
        label_key: "button.save_settings",
        label: None,
        visible: Some(|save, _| {
            matches!(save.current_kind, Some(SaveKind::Free | SaveKind::Puzzle))
        }),
        on_click: |commands| {
            commands.run_system_cached(open_save_settings);
        },
    },
    PauseMenuButton {
        label_key: "button.back_to_main_menu",
        label: None,
        visible: None,
        on_click: |commands| {
            commands.run_system_cached(return_to_main_menu);
        },
    },
];

/// 继续游戏只需要导航状态。
fn resume_playing(mut navigation: ResMut<UiNavigation>) {
    navigation.close_pause();
}

/// 模式按钮决定是否确认，实际世界事务由会话层执行。
fn toggle_builder_mode(
    view: UiContext,
    builder: Res<BuilderMode>,
    solution: Res<SolutionState>,
    mut commands: Commands,
) {
    if solution.entry != WorldEntryMode::EditPuzzle {
        return;
    }
    if *builder == BuilderMode::Edit {
        commands.write_message(session::BeginSolutionPlay);
    } else {
        let _scope = view.enter();
        ui.open_confirm_then(&mut commands, save_before_edit_spec(), on_save_before_edit);
    }
}

/// 保存按钮只选择确认流程和会话请求。
fn save_world(view: UiContext, save: Res<SaveState>, mut commands: Commands) {
    let _scope = view.enter();
    if puzzle_save_needs_confirm(&save) {
        open_save_puzzle_confirm(&mut commands);
    } else {
        session::save_current_world(&mut commands);
    }
}

/// 为另存谜题按钮打开命名输入。
fn save_as_puzzle(view: UiContext, mut commands: Commands) {
    let _scope = view.enter();
    open_save_as_new_puzzle_prompt(&mut commands);
}

/// 为导出谜题按钮打开命名输入。
fn export_puzzle(view: UiContext, mut commands: Commands) {
    let _scope = view.enter();
    open_export_as_puzzle_prompt(&mut commands);
}

/// 重置按钮只打开确认框。
fn confirm_reset_solution(view: UiContext, mut commands: Commands) {
    let _scope = view.enter();
    ui.open_confirm_then(&mut commands, reset_solution_spec(), on_reset_solution);
}

/// 设置按钮挂载全局设置页。
fn open_settings(root: Option<Res<PlayingUiRootEntity>>, mut commands: Commands) {
    ui.mount_settings(
        &mut commands,
        root.as_ref().map(|root| root.0),
        UiPanelContext::SettingsFromPause,
    );
}

/// 存档设置按钮只请求页面挂载。
fn open_save_settings(root: Option<Res<PlayingUiRootEntity>>, mut commands: Commands) {
    ui.mount_save_settings(&mut commands, root.as_ref().map(|root| root.0));
}

/// 返回按钮只读取未保存标记，保存和换档由会话层处理。
fn return_to_main_menu(view: UiContext, solution: Res<SolutionState>, mut commands: Commands) {
    let _scope = view.enter();
    if solution.dirty {
        ui.open_confirm_then(&mut commands, return_to_main_spec(), on_return_to_main);
    } else {
        session::exit_to_main_menu(&mut commands, false);
    }
}

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                dispatch_pause_menu_clicks.in_set(crate::game::schedule::GameSet::Menus),
                sync_pause_menu_buttons
                    .run_if(|ui_navigation: Res<UiNavigation>| ui_navigation.is_paused())
                    .after(crate::game::ui::update_localized_ui)
                    .in_set(crate::game::schedule::GameSet::UiFeat),
                sync_playing_overlay_mounts.in_set(crate::game::schedule::GameSet::Menus),
            ),
        );
    }
}

pub fn spawn_pause_menu_buttons(panel: &mut ChildSpawnerCommands) {
    for (index, button) in PAUSE_MENU_BUTTONS.iter().enumerate() {
        spawn_menu_button(
            panel,
            MenuButtonSet::PauseMenu,
            index as u8,
            button.label_key,
        );
    }
}

/// 分发点击只检查页面可用性和按钮显隐，不取得被点击业务的写资源。
fn dispatch_pause_menu_clicks(
    mut clicks: MessageReader<MenuButtonClick>,
    mode: Res<State<GameMode>>,
    navigation: Res<UiNavigation>,
    save: Res<SaveState>,
    solution: Res<SolutionState>,
    busy: Res<SessionBusy>,
    mut commands: Commands,
) {
    for click in clicks.read() {
        if *mode.get() != GameMode::Playing
            || !navigation.is_paused()
            || busy.is_busy()
            || click.set != MenuButtonSet::PauseMenu
        {
            continue;
        }
        let Some(button) = PAUSE_MENU_BUTTONS.get(click.index as usize) else {
            continue;
        };
        if button
            .visible
            .is_some_and(|visible| !visible(&save, &solution))
        {
            continue;
        }
        (button.on_click)(&mut commands);
        // 页面动作每帧提交一次，避免退出后继续消费排队的菜单点击。
        break;
    }
}

/// 根据只读展示数据更新暂停菜单。
fn sync_pause_menu_buttons(
    ui_context: UiContext,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    mut buttons: Query<(&MenuButtonMarker, &Children, &mut Node), With<Button>>,
    mut texts: Query<&mut Text>,
    added: Query<(), Added<MenuButtonMarker>>,
) {
    let _ui_scope = ui_context.enter();
    let labels_dirty = save_state.is_changed() || ui_context.locale_changed() || !added.is_empty();
    for (marker, children, mut node) in &mut buttons {
        if marker.set != MenuButtonSet::PauseMenu {
            continue;
        }
        let Some(button) = PAUSE_MENU_BUTTONS.get(marker.index as usize) else {
            continue;
        };
        let next = if button
            .visible
            .is_none_or(|visible| visible(&save_state, &solution_state))
        {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != next {
            node.display = next;
        }
        if !labels_dirty {
            continue;
        }
        let label = match button.label {
            Some(label) => label(&save_state),
            None => i18n.t(button.label_key),
        };
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                if text.0 != label {
                    text.0 = label.clone();
                }
            }
        }
    }
}
