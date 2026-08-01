use bevy::prelude::*;

/// 背包标题文字
#[derive(Component)]
pub struct InventoryTitleText;

/// Free 背包页签按钮
#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct InventoryTabButton(pub crate::game::ui::types::FreeInventoryTab);
