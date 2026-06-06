use super::TeleportExitBlock;

use crate::game::blocks::traits::BlockUi;
use crate::game::state::UiPanelId;

impl BlockUi for TeleportExitBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Teleport)
    }
}

// Teleport panel UI is owned by teleport_entrance/ui.rs.
