use super::*;

pub(super) fn ui_panel(_block: &TeleportExitBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Teleport)
}
