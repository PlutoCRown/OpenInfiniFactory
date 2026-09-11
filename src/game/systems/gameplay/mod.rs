//! 玩法输入与放置/悬停系统模块

mod aim_focus;
mod clipboard;
mod edit_ops;
mod edit_timing;
mod hover;
mod input;
mod placement;
mod play_gate;
mod rules;
mod selection;

pub use aim_focus::{AimBlockInfo, AimFocus, sync_aim_focus};
pub use clipboard::{BlockSettingsClipboard, SelectionToolSwap, clipboard_input};
pub use edit_timing::EditBatchTiming;
pub use hover::{
    apply_fov, draw_hover_structure_bounds, sync_factory_activity_debug_overlays, update_hover,
};
pub use input::gameplay_input;
pub use placement::placement_input;
pub use play_gate::GameplayPlayGate;
pub use selection::sync_edit_bounds_overlays;

/// 玩法插件：拥有编辑输入、目标选择和玩家操作的资源与执行顺序。
pub struct GameplayPlugin;

impl bevy::prelude::Plugin for GameplayPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use crate::game::cameras::{sync_gameplay_render_rate, sync_gameplay_view_image_size};
        use crate::game::edit_history::{EditHistory, edit_history_input};
        use crate::game::player::controller::{camera_look, camera_move, sync_cursor_grab};
        use crate::game::schedule::GameSet;
        use crate::game::{input, world};
        use bevy::prelude::*;
        app.init_resource::<crate::game::state::PlacementState>()
            .init_resource::<crate::game::world::rendering::HoverStructureBounds>()
            .init_resource::<AimFocus>()
            .init_resource::<EditBatchTiming>()
            .init_resource::<EditHistory>()
            .init_resource::<BlockSettingsClipboard>()
            .init_resource::<SelectionToolSwap>()
            .add_message::<placement::PlayerTeleportRequest>()
            .add_systems(
                Update,
                (
                    sync_cursor_grab.before(input::gather_gameplay_input),
                    camera_look.after(input::gather_gameplay_input),
                    gameplay_input,
                )
                    .chain()
                    .in_set(crate::game::schedule::GameSet::InputGather),
            )
            .add_systems(
                Update,
                camera_move.in_set(crate::game::schedule::GameSet::PlayerMove),
            )
            .add_systems(
                Update,
                (
                    sync_gameplay_view_image_size,
                    sync_gameplay_render_rate,
                    world::rendering::sync_shadow_settings,
                    world::rendering::sync_ssao_settings,
                    world::rendering::sync_vsync_settings,
                    world::rendering::sync_window_mode_settings,
                )
                    .in_set(crate::game::schedule::GameSet::Menus),
            )
            .add_systems(
                Update,
                edit_history_input
                    .after(GameSet::Hover)
                    .before(placement_input),
            )
            .add_systems(
                Update,
                clipboard_input
                    .after(GameSet::Hover)
                    .before(placement_input),
            )
            .add_systems(
                Update,
                update_hover.in_set(crate::game::schedule::GameSet::Hover),
            )
            .add_systems(
                Update,
                sync_aim_focus.after(update_hover).in_set(GameSet::Hover),
            )
            .add_systems(
                Update,
                sync_factory_activity_debug_overlays
                    .after(update_hover)
                    .in_set(GameSet::Hover),
            )
            .add_systems(
                Update,
                (placement_input, placement::apply_player_teleport_request)
                    .chain()
                    .in_set(crate::game::schedule::GameSet::Placement),
            )
            .add_systems(
                Update,
                sync_edit_bounds_overlays
                    .after(placement_input)
                    .in_set(crate::game::schedule::GameSet::Menus),
            )
            .add_systems(
                Update,
                (apply_fov, draw_hover_structure_bounds)
                    .in_set(crate::game::schedule::GameSet::View),
            );
    }
}
