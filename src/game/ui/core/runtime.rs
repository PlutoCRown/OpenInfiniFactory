use bevy::prelude::*;

use crate::game::state::UiPanelId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPanelContext {
    SettingsFromStartMenu,
    SettingsFromPause,
    Block { pos: IVec3 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelSession {
    pub panel: UiPanelId,
    pub context: UiPanelContext,
}

#[derive(Resource, Default)]
pub struct UiRuntime {
    stack: Vec<UiPanelSession>,
}

impl UiRuntime {
    pub fn open(&mut self, panel: UiPanelId, context: UiPanelContext) {
        self.stack.retain(|session| session.panel != panel);
        self.stack.push(UiPanelSession { panel, context });
    }

    pub fn open_block(&mut self, panel: UiPanelId, pos: IVec3) {
        self.open(panel, UiPanelContext::Block { pos })
    }

    pub fn close_active(&mut self) {
        self.stack.pop();
    }

    pub fn close_panel(&mut self, panel: UiPanelId) {
        self.stack.retain(|session| session.panel != panel);
    }

    pub fn close_current(&mut self) {
        self.close_active();
    }

    pub fn active(&self) -> Option<UiPanelSession> {
        self.stack.last().copied()
    }

    pub fn active_panel(&self) -> Option<UiPanelId> {
        self.active().map(|session| session.panel)
    }

    pub fn is_settings_open(&self) -> bool {
        self.active_panel().is_some_and(UiPanelId::is_settings)
    }

    pub fn blocks_gameplay(&self) -> bool {
        self.active_panel()
            .is_some_and(UiPanelId::is_blocking_gameplay)
    }

    pub fn active_block_pos(&self) -> Option<IVec3> {
        match self.active()?.context {
            UiPanelContext::Block { pos } => Some(pos),
            _ => None,
        }
    }

    pub fn panel_layer(&self, panel: UiPanelId) -> Option<usize> {
        self.stack.iter().position(|session| session.panel == panel)
    }

    pub fn top_modal_layer(&self) -> Option<usize> {
        self.stack
            .iter()
            .rposition(|session| session.panel.is_blocking_gameplay())
    }

}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelBinding(pub UiPanelId);
