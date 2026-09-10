//! 游玩态覆盖层：背包常驻只显隐；暂停菜单按需挂载

use bevy::prelude::*;

use crate::game::state::GameMode;
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;
use crate::game::ui::core::host::{PlayingUiRootEntity, UiHostMountRoot};
use crate::game::ui::core::{UiMountState, UiNavigation};
use crate::game::ui::screens::spawn_pause_panel;

/// 同步暂停菜单挂载；背包由 setup_ui_navigation 常驻，只靠 Display 显隐
pub fn sync_playing_overlay_mounts(
    _ui_thread: crate::game::ui::access::UiMainThread,
    mode: Res<State<GameMode>>,
    ui_navigation: Res<UiNavigation>,
    root: Option<Res<PlayingUiRootEntity>>,
    mut mounts: ResMut<UiMountState>,
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

    match (ui_navigation.is_paused(), mounts.pause) {
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
        app.add_systems(
            Update,
            sync_playing_overlay_mounts
                .in_set(UiAccessScope)
                .after(PerfScope::Placement)
                .before(PerfScope::Menus),
        );
    }
}
