//! 游玩态按需挂载：背包 / 暂停菜单（不显示则不存在实体）

use bevy::prelude::*;

use crate::game::state::{GameMode, PlayingUiState};
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;
use crate::game::ui::core::host::{PlayingUiRootEntity, UiHostMountRoot};
use crate::game::ui::screens::{spawn_inventory_panel, spawn_pause_panel};

/// 记录当前已挂载的游玩覆盖层根实体
#[derive(Resource, Default)]
pub struct PlayingOverlayMounts {
    pub inventory: Option<Entity>,
    pub pause: Option<Entity>,
}

/// 按 PlayingUiState 同步挂载/卸载背包与暂停菜单
pub fn sync_playing_overlay_mounts(
    _ui_thread: crate::game::ui::access::UiMainThread,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    root: Option<Res<PlayingUiRootEntity>>,
    mut mounts: ResMut<PlayingOverlayMounts>,
    mut commands: Commands,
) {
    if *mode.get() != GameMode::Playing {
        if mounts.inventory.take().is_some() || mounts.pause.take().is_some() {
            // Playing 根已由 on_exit 销毁；只清资源记录
            mounts.inventory = None;
            mounts.pause = None;
        }
        return;
    }
    let Some(root) = root.map(|r| r.0) else {
        return;
    };

    match (playing_ui.inventory_open, mounts.inventory) {
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
                    .with_children(|container| {
                        spawn_inventory_panel(container);
                    })
                    .id(),
                );
            });
            mounts.inventory = entity;
        }
        (false, Some(entity)) => {
            commands.entity(entity).despawn();
            mounts.inventory = None;
        }
        _ => {}
    }

    match (playing_ui.paused, mounts.pause) {
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
                    .with_children(|container| {
                        spawn_pause_panel(container);
                    })
                    .id(),
                );
            });
            mounts.pause = entity;
        }
        (false, Some(entity)) => {
            commands.entity(entity).despawn();
            mounts.pause = None;
        }
        _ => {}
    }
}

pub struct PlayingOverlaysPlugin;

impl Plugin for PlayingOverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayingOverlayMounts>().add_systems(
            Update,
            sync_playing_overlay_mounts
                .in_set(UiAccessScope)
                .after(PerfScope::Input)
                .before(PerfScope::Menus),
        );
    }
}
