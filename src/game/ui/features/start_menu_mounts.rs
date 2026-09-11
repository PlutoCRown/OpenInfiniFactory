//! 主菜单 / 存档列表按需挂载

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::session::SessionBusy;
use crate::game::state::GameMode;
use crate::game::ui::access::{i18n, ui_icons};
use crate::game::ui::core::host::{UiHostMountRoot, UiRootEntity};
use crate::game::ui::core::{StartMenuPage, UiMountState, UiNavigation};
use crate::game::ui::features::save::save_list_title;
use crate::game::ui::features::save::types::SaveListRenderState;
use crate::game::ui::features::session_busy::spawn_session_busy_overlay;
use crate::game::ui::screens::{
    SaveListSpawnCtx, save_list_panel_size, spawn_main_menu, spawn_save_list,
};

/// 按当前屏同步挂载主菜单与存档列表
pub fn sync_start_menu_mounts(
    ui_context: crate::game::ui::access::UiContext,
    mode: Res<State<GameMode>>,
    navigation: Res<UiNavigation>,
    busy: Res<SessionBusy>,
    root: Option<Res<UiRootEntity>>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut mounts: ResMut<UiMountState>,
    mut save_list_render: ResMut<SaveListRenderState>,
    mut commands: Commands,
) {
    let _ui_scope = ui_context.enter();
    if *mode.get() != GameMode::StartMenu {
        for entity in [mounts.main_menu.take(), mounts.save_list.take()]
            .into_iter()
            .flatten()
        {
            commands.entity(entity).despawn();
        }
        // 忙碌遮罩留到 busy 结束：进 Playing 的过渡帧仍可能靠上一帧像素
        if !busy.is_busy() {
            if let Some(entity) = mounts.session_busy.take() {
                commands.entity(entity).despawn();
            }
        }
        // 卸掉存档列表后清渲染缓存，否则下次进列表会跳过行重建
        *save_list_render = SaveListRenderState::default();
        return;
    }
    let Some(root) = root.map(|r| r.0) else {
        return;
    };

    if mounts.session_busy.is_none() {
        let busy_now = *busy;
        let mut entity = None;
        commands.entity(root).with_children(|root| {
            entity = Some(
                root.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    UiHostMountRoot,
                    Pickable::IGNORE,
                ))
                .with_children(|c| {
                    spawn_session_busy_overlay(c, busy_now);
                })
                .id(),
            );
        });
        mounts.session_busy = entity;
    }

    let want_main = navigation.start_menu() == StartMenuPage::Main;
    let want_save = navigation.start_menu() == StartMenuPage::SaveList;

    match (want_main, mounts.main_menu) {
        (true, None) => {
            let mut entity = None;
            commands.entity(root).with_children(|root| {
                entity = Some(
                    root.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        UiHostMountRoot,
                        Pickable::IGNORE,
                    ))
                    .with_children(|c| {
                        spawn_main_menu(c);
                    })
                    .id(),
                );
            });
            mounts.main_menu = entity;
        }
        (false, Some(entity)) => {
            commands.entity(entity).despawn();
            mounts.main_menu = None;
        }
        _ => {}
    }

    match (want_save, mounts.save_list) {
        (true, None) => {
            let (win_w, win_h) = windows
                .single()
                .map(|window| (window.width(), window.height()))
                .unwrap_or((1280.0, 720.0));
            let (panel_w, panel_h) = save_list_panel_size(win_w, win_h, ui_scale.0);
            let window_aspect = (win_w / win_h.max(1.0)).max(0.5);
            // 须在 with_children 延后执行前解析完（延后阶段已离开 UI 命令提交）
            let spawn_ctx = SaveListSpawnCtx {
                title: save_list_title(),
                puzzle_heading: i18n.t("save.title.select_puzzle_list"),
                solution_heading: i18n.t("save.title.select_solution"),
                icons: ui_icons(),
            };
            let mut entity = None;
            commands.entity(root).with_children(|root| {
                entity = Some(
                    root.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        UiHostMountRoot,
                        Pickable::IGNORE,
                    ))
                    .with_children(|c| {
                        spawn_save_list(c, panel_w, panel_h, window_aspect, spawn_ctx);
                    })
                    .id(),
                );
            });
            mounts.save_list = entity;
            // 新挂载的行容器是空的，必须丢弃上次的 keys 缓存
            *save_list_render = SaveListRenderState::default();
            save_list_render.paint_buttons = true;
        }
        (false, Some(entity)) => {
            commands.entity(entity).despawn();
            mounts.save_list = None;
            *save_list_render = SaveListRenderState::default();
        }
        _ => {}
    }
}
