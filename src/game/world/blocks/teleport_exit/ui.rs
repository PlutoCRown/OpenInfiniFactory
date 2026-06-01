use super::*;

pub(super) fn default_settings(
    _block: &TeleportExitBlock,
    pos: bevy::prelude::IVec3,
) -> Option<BlockSettings> {
    Some(BlockSettings::Teleport(TeleportSettings::unnamed(pos)))
}

pub(super) fn ui_panel(_block: &TeleportExitBlock) -> Option<UiPanelId> {
    Some(UiPanelId::Teleport)
}
