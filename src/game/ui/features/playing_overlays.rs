//! 游玩态覆盖层：背包常驻只显隐；暂停菜单按需挂载

use bevy::prelude::*;

use crate::game::state::{GameMode, PlayingUiState};
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;
use crate::game::ui::core::host::{PlayingUiRootEntity, UiHostMountRoot};
use crate::game::ui::screens::spawn_pause_panel;

/// 记录当前已挂载的游玩覆盖层根实体
#[derive(Resource, Default)]
pub struct PlayingOverlayMounts {
    pub inventory: Option<Entity>,
    pub pause: Option<Entity>,
}

/// 同步暂停菜单挂载；背包由 setup_playing_ui 常驻，只靠 Display 显隐
pub fn sync_playing_overlay_mounts(
    _ui_thread: crate::game::ui::access::UiMainThread,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    root: Option<Res<PlayingUiRootEntity>>,
    mut mounts: ResMut<PlayingOverlayMounts>,
    mut commands: Commands,
) {
    if *mode.get() != GameMode::Playing {
        // Playing 根已由 on_exit 销毁；只清资源记录
        mounts.inventory = None;
        mounts.pause = None;
        return;
    }
    let Some(root) = root.map(|r| r.0) else {
        return;
    };

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
