use bevy::picking::pointer::PointerId;
use bevy::prelude::*;

use crate::game::ui::types::InventoryItem;

/// 背包标题文字
#[derive(Component)]
pub struct InventoryTitleText;

/// Free 背包页签按钮
#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct InventoryTabButton(pub crate::game::ui::types::FreeInventoryTab);

/// 触控背包的点选提示与拖动物品状态
#[derive(Resource, Default)]
pub struct TouchInventoryState {
    pub selected_backpack: Option<(usize, InventoryItem)>,
    pub drag_pointer: Option<Vec2>,
    pub drag_pointer_id: Option<PointerId>,
    pub drag_grab_offset: Vec2,
    pub dragging: bool,
}

impl TouchInventoryState {
    /// 清除触控背包的临时交互状态
    pub fn clear(&mut self) {
        self.selected_backpack = None;
        self.drag_pointer = None;
        self.drag_pointer_id = None;
        self.drag_grab_offset = Vec2::ZERO;
        self.dragging = false;
    }
}
