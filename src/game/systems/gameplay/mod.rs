//! 玩法输入与放置/悬停系统模块

mod aim_focus;
mod clipboard;
mod edit_ops;
mod hover;
mod input;
mod placement;
mod rules;
mod selection;

pub use aim_focus::{AimBlockInfo, AimFocus, sync_aim_focus};
pub use clipboard::{BlockSettingsClipboard, SelectionToolSwap, clipboard_input};
pub use hover::{
    apply_fov, draw_hover_structure_bounds, sync_factory_activity_debug_overlays, update_hover,
};
pub use input::gameplay_input;
pub use placement::placement_input;
pub use selection::sync_edit_bounds_overlays;
