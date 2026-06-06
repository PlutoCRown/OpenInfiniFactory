use bevy::prelude::*;

use crate::game::state::UiPanelId;

/// Which block-property dropdown is open: `(panel, slot)` where `slot` is block-defined.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub struct OpenBlockPanelDropdown(pub Option<(UiPanelId, u8)>);

impl OpenBlockPanelDropdown {
    pub fn toggle(&mut self, panel: UiPanelId, slot: u8) {
        let next = Some((panel, slot));
        self.0 = if self.0 == next { None } else { next };
    }

    pub fn is_open(&self, panel: UiPanelId, slot: u8) -> bool {
        self.0 == Some((panel, slot))
    }

    pub fn close(&mut self) {
        self.0 = None;
    }

    pub fn close_panel(&mut self, panel: UiPanelId) {
        if self.0.is_some_and(|(open, _)| open == panel) {
            self.0 = None;
        }
    }
}
