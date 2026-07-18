use bevy::prelude::*;

use crate::game::state::StartMenuScreen;

#[derive(Component)]
pub struct PanelWindow;

/// 并排流式面板：初始由父级 flex 居中，拖动后切 Absolute；隐藏时还原
#[derive(Component)]
pub struct PanelFlowLayout;

#[derive(Component, Default)]
pub struct PanelPosition {
    pub dragged: bool,
}

#[derive(Component)]
pub struct PanelTitleBar;

#[derive(Component)]
pub struct PanelCloseButton;

#[derive(Resource, Default)]
pub struct PanelDragState {
    pub panel: Option<Entity>,
    /// Pointer position minus panel top-left when the drag started (logical px).
    pub grab_offset: Vec2,
}

impl PanelDragState {
    pub fn clear(&mut self) {
        self.panel = None;
        self.grab_offset = Vec2::ZERO;
    }
}

#[derive(Resource, Default)]
pub struct UiHoverState {
    pub entity: Option<Entity>,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum PanelVisibility {
    StartMenuScreen(StartMenuScreen),
    PauseMenu,
    Inventory,
    SettingsTab(crate::game::ui::features::settings::types::SettingsTab),
    ConfirmDialog,
}
