use bevy::prelude::*;

use crate::game::edit_history::EditHistory;
use crate::game::player::controller::{FlyCamera, capture_player_save};
use crate::game::state::{GameMode, StartMenuScreen};

use super::busy::SessionBusy;
#[cfg(not(target_arch = "wasm32"))]
use super::cover::{CoverScreenshotComplete, begin_cover_capture};
use super::cover::{DeferredMainMenuExit, PendingMainMenuExit, should_capture_cover};
use super::messages::ExitToMainMenu;
use super::world_access::{PlayingWorldParams, SessionStateParams};
use super::world_ops::{
    exit_to_main_menu, save_current_world, save_current_world_invalidate_solutions,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::game::cameras::GameplayViewImage;

/// 收到退出请求：不保存则立刻回菜单；要保存则先亮「保存中」再写档
pub fn handle_exit_to_main_menu(
    mut requests: MessageReader<ExitToMainMenu>,
    mut busy: ResMut<SessionBusy>,
    mut pending_exit: ResMut<PendingMainMenuExit>,
) {
    for request in requests.read() {
        if request.save_first {
            *busy = SessionBusy::Saving;
            pending_exit.deferred = Some(DeferredMainMenuExit {
                save_first: true,
                invalidate_solutions: request.invalidate_solutions,
                hold_frames: 2,
            });
        } else {
            // 不保存退出：不显示「保存中」，也不截封面
            pending_exit.deferred = Some(DeferredMainMenuExit {
                save_first: false,
                invalidate_solutions: false,
                hold_frames: 0,
            });
        }
    }
}

/// 消耗延后的退出：可选存档/封面截图，再回主菜单
pub fn process_deferred_main_menu_exit(
    mut playing: PlayingWorldParams,
    mut session: SessionStateParams,
    player: Query<(&FlyCamera, &Transform)>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut pending_exit: ResMut<PendingMainMenuExit>,
    mut edit_history: ResMut<EditHistory>,
    mut busy: ResMut<SessionBusy>,
    #[cfg(not(target_arch = "wasm32"))] view_image: Option<Res<GameplayViewImage>>,
) {
    if pending_exit.waiting_cover {
        return;
    }
    let Some(mut deferred) = pending_exit.deferred else {
        return;
    };
    if deferred.hold_frames > 0 {
        deferred.hold_frames -= 1;
        pending_exit.deferred = Some(deferred);
        return;
    }
    pending_exit.deferred = None;

    let player_save = player
        .single()
        .ok()
        .map(|(camera, transform)| capture_player_save(camera, transform));
    if deferred.save_first {
        if deferred.invalidate_solutions {
            save_current_world_invalidate_solutions(
                &playing.world,
                &session.inventory,
                &mut session.save_state,
                &mut session.solution_state,
                &session.simulation,
                player_save.clone(),
            );
        } else {
            save_current_world(
                &playing.world,
                &session.inventory,
                &mut session.save_state,
                &mut session.solution_state,
                &session.simulation,
                player_save.clone(),
            );
        }
        // 封面只在本次真正写档后更新；不保存退出沿用磁盘上的旧封面
        if should_capture_cover(session.save_state.current.as_ref()) {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(view_image) = view_image.as_ref() {
                begin_cover_capture(
                    &mut playing.commands,
                    session.save_state.current.as_ref().unwrap(),
                    view_image,
                );
                pending_exit.waiting_cover = true;
                *busy = SessionBusy::Saving;
                return;
            }
        }
    }
    edit_history.clear();
    exit_to_main_menu(
        &mut playing,
        &mut session,
        &mut next_state,
        &mut start_menu_screen,
    );
    if deferred.save_first {
        // OnExit(Playing) 拆景在下一帧；等回 StartMenu 后再清 busy
        pending_exit.release_busy_after_menu = true;
    } else {
        *busy = SessionBusy::None;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn finish_pending_main_menu_exit(
    mut complete: MessageReader<CoverScreenshotComplete>,
    mut playing: PlayingWorldParams,
    mut session: SessionStateParams,
    mut next_state: ResMut<NextState<GameMode>>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut pending_exit: ResMut<PendingMainMenuExit>,
    mut edit_history: ResMut<EditHistory>,
) {
    if !pending_exit.waiting_cover {
        return;
    }
    for _ in complete.read() {
        pending_exit.waiting_cover = false;
        edit_history.clear();
        exit_to_main_menu(
            &mut playing,
            &mut session,
            &mut next_state,
            &mut start_menu_screen,
        );
        pending_exit.release_busy_after_menu = true;
    }
}

/// OnExit(Playing) 完成并回到主菜单后再关掉「保存中」
pub fn release_session_busy_after_menu(
    mut pending_exit: ResMut<PendingMainMenuExit>,
    mut busy: ResMut<SessionBusy>,
    mode: Res<State<GameMode>>,
) {
    if !pending_exit.release_busy_after_menu || *mode.get() != GameMode::StartMenu {
        return;
    }
    pending_exit.release_busy_after_menu = false;
    if *busy == SessionBusy::Saving {
        *busy = SessionBusy::None;
    }
}
