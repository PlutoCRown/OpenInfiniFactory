use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::ui::access::UiMainThread;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;

use super::panel_state::OpenBlockPanelDropdown;

/// 块面板下拉刷新系统的共享依赖（图标槽、材料列表、颜色选择等）
#[derive(SystemParam)]
pub struct BlockPanelDropdownDeps<'w, 's> {
    pub _ui_thread: UiMainThread,
    pub ui_runtime: Res<'w, UiRuntime>,
    pub open_dropdown: Res<'w, OpenBlockPanelDropdown>,
    pub world: Res<'w, WorldBlocks>,
    pub block_icons: Option<Res<'w, BlockIconAssets>>,
    pub commands: Commands<'w, 's>,
    pub windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}
