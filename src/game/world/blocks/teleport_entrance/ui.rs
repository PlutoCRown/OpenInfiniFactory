use super::*;

pub(super) fn default_settings(
    _block: &TeleportEntranceBlock,
    pos: bevy::prelude::IVec3,
) -> Option<BlockSettings> {
    Some(BlockSettings::Teleport(TeleportSettings::unnamed(pos)))
}

pub(super) fn ui_panel(_block: &TeleportEntranceBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Teleport)
}
