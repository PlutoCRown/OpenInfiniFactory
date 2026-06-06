use super::RollerBlock;

use crate::game::blocks::traits::BlockUi;
use crate::game::state::UiPanelId;

impl BlockUi for RollerBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Labeler)
    }
}

// Labeler panel UI is owned by stamper/ui.rs; roller reuses the same panel and actions.
