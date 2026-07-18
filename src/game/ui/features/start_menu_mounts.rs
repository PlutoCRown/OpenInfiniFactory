//! 主菜单 / 存档列表按需挂载

use bevy::prelude::*;

use crate::game::state::{GameMode, StartMenuScreen};
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;
use crate::game::ui::core::host::{UiHostMountRoot, UiRootEntity};
use crate::game::ui::features::session_busy::spawn_session_busy_overlay;
use crate::game::ui::screens::{spawn_main_menu, spawn_save_list};

/// 菜单态已挂载的覆盖层
#[derive(Resource, Default)]
pub struct StartMenuMounts {
    pub main: Option<Entity>,
    pub save_list: Option<Entity>,
    pub session_busy: Option<Entity>,
}

/// 按当前屏同步挂载主菜单与存档列表
pub fn sync_start_menu_mounts(
    _ui_thread: crate::game::ui::access::UiMainThread,
    mode: Res<State<GameMode>>,
    screen: Res<StartMenuScreen>,
    root: Option<Res<UiRootEntity>>,
    mut mounts: ResMut<StartMenuMounts>,
    mut commands: Commands,
) {
    if *mode.get() != GameMode::StartMenu {
        for entity in [
            mounts.main.take(),
            mounts.save_list.take(),
            mounts.session_busy.take(),
        ]
        .into_iter()
        .flatten()
        {
            commands.entity(entity).despawn();
        }
        return;
    }
    let Some(root) = root.map(|r| r.0) else {
        return;
    };

    if mounts.session_busy.is_none() {
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
                    spawn_session_busy_overlay(c);
                })
                .id(),
            );
        });
        mounts.session_busy = entity;
    }

    let want_main = *screen == StartMenuScreen::Main;
    let want_save = *screen == StartMenuScreen::SaveList;

    match (want_main, mounts.main) {
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
            mounts.main = entity;
        }
        (false, Some(entity)) => {
            commands.entity(entity).despawn();
            mounts.main = None;
        }
        _ => {}
    }

    match (want_save, mounts.save_list) {
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
                        spawn_save_list(c);
                    })
                    .id(),
                );
            });
            mounts.save_list = entity;
        }
        (false, Some(entity)) => {
            commands.entity(entity).despawn();
            mounts.save_list = None;
        }
        _ => {}
    }
}

pub struct StartMenuMountsPlugin;

impl Plugin for StartMenuMountsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StartMenuMounts>().add_systems(
            Update,
            sync_start_menu_mounts
                .in_set(UiAccessScope)
                .after(PerfScope::Input)
                .before(PerfScope::Menus),
        );
    }
}
