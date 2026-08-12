//! 本地玩家会话：放置/背包/模式/撤销/工具临时态
//!
//! 单机仍用 Resource；读写经 LocalPlayer(Mut)，便于日后改为每玩家实体组件。

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::edit_history::EditHistory;
use crate::game::state::{BuilderMode, PlacementState};
use crate::game::ui::{CarriedItem, FreeInventoryTab, InventoryItem, InventoryItems};
use crate::game::world::grid::BlockSettings;

/// 系统方块配置剪贴板
#[derive(Resource, Default)]
pub struct BlockSettingsClipboard(pub Option<BlockSettings>);

/// Ctrl+X 临时换成选区工具时，记下被替换的快捷栏物品（含空槽）
#[derive(Resource, Default)]
pub struct SelectionToolSwap {
    pub displaced: Option<Option<InventoryItem>>,
}

/// 本地玩家会话（只读）
#[derive(SystemParam)]
pub struct LocalPlayer<'w> {
    pub placement: Res<'w, PlacementState>,
    pub inventory: Res<'w, InventoryItems>,
    pub carried: Res<'w, CarriedItem>,
    pub free_inventory_tab: Res<'w, FreeInventoryTab>,
    pub builder_mode: Res<'w, BuilderMode>,
    pub edit_history: Res<'w, EditHistory>,
    pub clipboard: Res<'w, BlockSettingsClipboard>,
    pub tool_swap: Res<'w, SelectionToolSwap>,
}

/// 本地玩家会话（可变）
#[derive(SystemParam)]
pub struct LocalPlayerMut<'w> {
    pub placement: ResMut<'w, PlacementState>,
    pub inventory: ResMut<'w, InventoryItems>,
    pub carried: ResMut<'w, CarriedItem>,
    pub free_inventory_tab: ResMut<'w, FreeInventoryTab>,
    pub builder_mode: ResMut<'w, BuilderMode>,
    pub edit_history: ResMut<'w, EditHistory>,
    pub clipboard: ResMut<'w, BlockSettingsClipboard>,
    pub tool_swap: ResMut<'w, SelectionToolSwap>,
}
