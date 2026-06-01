use super::*;

pub(super) fn ui_panel(_block: &TeleportEntranceBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Teleport)
}
