use bevy::prelude::*;

use crate::game::blocks::BlockKind;
use crate::shared::save::{FactoryBlockFilterMode, SaveSettingsData, SaveSlot};

/// 存档设置页中的可点击动作。
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveSettingsAction {
    EditLightPosition,
    EditLightDirection,
    EditLightIntensity,
    CaptureLightPose,
    EditSolutionSpawn,
    CaptureSolutionSpawn,
    SetFactoryFilterMode(FactoryBlockFilterMode),
    OpenFactoryPicker,
    CloseFactoryPicker,
    ToggleFactoryBlock(BlockKind),
    UploadSkybox,
}

/// 存档设置页的运行时状态；只在面板打开期间由 UI 维护。
#[derive(Resource, Default)]
pub struct SaveSettingsUiState {
    pub slot: Option<SaveSlot>,
    pub data: SaveSettingsData,
    pub skybox_bytes: Option<Vec<u8>>,
    pub edit_mode: bool,
    pub picker_open: bool,
}
