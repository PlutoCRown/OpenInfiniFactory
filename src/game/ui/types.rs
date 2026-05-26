use bevy::prelude::*;

use crate::game::state::BuilderMode;
use crate::game::world::blocks::{BlockKind, EDIT_BLOCKS, PLAY_BLOCKS};
use crate::shared::config::ConfigAction;

pub const HOTBAR_SLOTS: usize = 9;
pub(super) const BACKPACK_SLOTS: usize = 27;

#[derive(Component)]
pub struct HotbarText;

#[derive(Component)]
pub struct BackpackPanel;

#[derive(Component)]
pub struct InventoryTitle;

#[derive(Component)]
pub struct PausePanel;

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Component)]
pub struct SettingsStatusText;

#[derive(Component)]
pub struct SettingsGameplayGroup;

#[derive(Component)]
pub struct SettingsKeyBindingsGroup;

#[derive(Component)]
pub struct KeyBindingLabel;

#[derive(Component, Clone, Copy)]
pub struct KeyBindingButton(pub ConfigAction);

#[derive(Component)]
pub struct MainMenuPanel;

#[derive(Component)]
pub struct SaveListPanel;

#[derive(Component)]
pub struct SaveListTitle;

#[derive(Component)]
pub struct SaveListLabel;

#[derive(Component)]
pub struct CurrentSaveText;

#[derive(Component)]
pub struct Crosshair;

#[derive(Component)]
pub struct FovText;

#[derive(Component)]
pub struct UiScaleText;

#[derive(Component)]
pub struct PlaceSelectionModeText;

#[derive(Component)]
pub struct DeleteSelectionModeText;

#[derive(Component)]
pub struct SimulationText;

#[derive(Component)]
pub struct LocalizedText {
    pub key: &'static str,
}

#[derive(Component)]
pub struct LanguageText;

#[derive(Component, Clone, Copy)]
pub enum PauseAction {
    Resume,
    ToggleBuilderMode,
    SaveWorld,
    OpenSaveList,
    OpenSettings,
    BackToMainMenu,
    Quit,
}

#[derive(Component, Clone, Copy)]
pub enum SimulationAction {
    ToggleRun,
    Rollback,
}

#[derive(Component, Clone, Copy)]
pub enum MainMenuAction {
    NewWorld,
    OpenSaveList,
    Quit,
}

#[derive(Component, Clone, Copy)]
pub enum SaveListAction {
    Load(usize),
    Back,
}

#[derive(Component, Clone, Copy)]
pub enum SettingsAction {
    TabGameplay,
    TabKeyBindings,
    FovDown,
    FovUp,
    UiScaleDown,
    UiScaleUp,
    PlaceSelectionModeNext,
    DeleteSelectionModeNext,
    LanguageNext,
    Bind(ConfigAction),
    ResetDefaults,
    OpenFolder,
    Back,
}

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
pub enum SettingsTab {
    Gameplay,
    KeyBindings,
}

#[derive(Resource, Default)]
pub struct PendingKeyBind(pub Option<ConfigAction>);

impl Default for SettingsTab {
    fn default() -> Self {
        Self::Gameplay
    }
}

#[derive(Component)]
pub(crate) struct SlotLabel;

#[derive(Component)]
pub(crate) struct CarriedLabel;

#[derive(Component, Clone, Copy)]
pub(crate) struct InventorySlot {
    pub area: SlotArea,
    pub index: usize,
}

#[derive(Resource)]
pub struct InventoryItems {
    pub hotbar: [Option<BlockKind>; HOTBAR_SLOTS],
    pub(super) backpack: [Option<BlockKind>; BACKPACK_SLOTS],
}

impl Default for InventoryItems {
    fn default() -> Self {
        Self::for_mode(BuilderMode::default())
    }
}

impl InventoryItems {
    pub fn for_mode(mode: BuilderMode) -> Self {
        let blocks: &[BlockKind] = match mode {
            BuilderMode::Edit => &EDIT_BLOCKS,
            BuilderMode::Play => &PLAY_BLOCKS,
        };

        let mut hotbar = [None; HOTBAR_SLOTS];
        for (index, kind) in blocks.iter().take(HOTBAR_SLOTS).enumerate() {
            hotbar[index] = Some(*kind);
        }

        let mut backpack = [None; BACKPACK_SLOTS];
        for index in 0..BACKPACK_SLOTS {
            backpack[index] = Some(blocks[index % blocks.len()]);
        }

        Self { hotbar, backpack }
    }
}

#[derive(Resource)]
pub struct CarriedItem(Option<BlockKind>);

impl Default for CarriedItem {
    fn default() -> Self {
        Self(None)
    }
}

impl CarriedItem {
    pub fn clear(&mut self) {
        self.0 = None;
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SlotArea {
    Hotbar,
    Backpack,
}

impl CarriedItem {
    pub(super) fn item(&self) -> Option<BlockKind> {
        self.0
    }

    pub(super) fn item_mut(&mut self) -> &mut Option<BlockKind> {
        &mut self.0
    }
}
