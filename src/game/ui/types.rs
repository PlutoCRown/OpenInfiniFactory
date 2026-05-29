use bevy::prelude::*;

use crate::game::state::BuilderMode;
use crate::game::world::blocks::{BlockKind, EDIT_BLOCKS, PLAY_BLOCKS};
use crate::shared::config::{ConfigAction, ConfigSelectionMode};
use crate::shared::i18n::Language;

pub const HOTBAR_SLOTS: usize = 9;
pub(super) const BACKPACK_SLOTS: usize = 27;

#[derive(Component)]
pub struct HotbarText;

#[derive(Component)]
pub struct InGameHudStyle;

#[derive(Component)]
pub struct InGameHudVisibility;

#[derive(Component)]
pub struct BackpackPanel;

#[derive(Component)]
pub struct InventoryTitle;

#[derive(Component)]
pub struct PausePanel;

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Component)]
pub struct GeneratorPanel;

#[derive(Component)]
pub struct GeneratorPeriodText;

#[derive(Component)]
pub struct GeneratorMaterialText;

#[derive(Component)]
pub struct LabelerPanel;

#[derive(Component)]
pub struct LabelerColorText;

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

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsValueText(pub SettingsValue);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsValue {
    Fov,
    UiScale,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsSliderFill(pub SettingsSlider);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsSliderKnob(pub SettingsSlider);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownLabel(pub SettingsDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownList(pub SettingsDropdown);

#[derive(Component)]
pub struct ScrollContainer {
    pub offset: f32,
    pub max_offset: f32,
}

#[derive(Component)]
pub struct ScrollContent;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsSlider {
    Fov,
    UiScale,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsDropdown {
    Language,
    PlaceSelectionMode,
    DeleteSelectionMode,
}

#[derive(Component)]
pub struct SimulationText;

#[derive(Component)]
pub struct LocalizedText {
    pub key: &'static str,
}

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
    OpenSettings,
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
    FovSlider,
    UiScaleSlider,
    SetPlaceSelectionMode(ConfigSelectionMode),
    SetDeleteSelectionMode(ConfigSelectionMode),
    SetLanguage(Language),
    ToggleDropdown(SettingsDropdown),
    Bind(ConfigAction),
    ResetDefaults,
    OpenFolder,
    Back,
}

#[derive(Component, Clone, Copy)]
pub enum GeneratorAction {
    PeriodDown,
    PeriodUp,
    MaterialNext,
    Close,
}

#[derive(Component, Clone, Copy)]
pub enum LabelerAction {
    PreviousColor,
    NextColor,
    Close,
}

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
pub enum SettingsTab {
    Gameplay,
    KeyBindings,
}

#[derive(Resource, Default)]
pub struct PendingKeyBind(pub Option<ConfigAction>);

#[derive(Resource, Default)]
pub struct OpenSettingsDropdown(pub Option<SettingsDropdown>);

impl Default for SettingsTab {
    fn default() -> Self {
        Self::Gameplay
    }
}

#[derive(Component)]
pub(crate) struct SlotLabel;

#[derive(Component)]
pub(crate) struct CarriedLabel;

#[derive(Component)]
pub(crate) struct CarriedIcon;

#[derive(Component, Clone, Copy)]
pub(crate) struct InventorySlot {
    pub area: SlotArea,
    pub index: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum AreaKind {
    Selection,
}

impl AreaKind {
    pub fn name_key(self) -> &'static str {
        match self {
            Self::Selection => "area.selection",
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InventoryItem {
    Block(BlockKind),
    Area(AreaKind),
}

impl InventoryItem {
    pub fn name_key(self) -> &'static str {
        match self {
            Self::Block(kind) => kind.name_key(),
            Self::Area(kind) => kind.name_key(),
        }
    }

    pub fn block(self) -> Option<BlockKind> {
        match self {
            Self::Block(kind) => Some(kind),
            Self::Area(_) => None,
        }
    }

    pub fn area(self) -> Option<AreaKind> {
        match self {
            Self::Area(kind) => Some(kind),
            Self::Block(_) => None,
        }
    }
}

#[derive(Resource)]
pub struct InventoryItems {
    pub hotbar: [Option<InventoryItem>; HOTBAR_SLOTS],
    pub(super) backpack: [Option<InventoryItem>; BACKPACK_SLOTS],
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
            hotbar[index] = Some(InventoryItem::Block(*kind));
        }

        let mut backpack = [None; BACKPACK_SLOTS];
        for (index, kind) in blocks.iter().take(BACKPACK_SLOTS).enumerate() {
            backpack[index] = Some(InventoryItem::Block(*kind));
        }

        Self { hotbar, backpack }
    }
}

#[derive(Resource)]
pub struct CarriedItem(Option<InventoryItem>);

impl Default for CarriedItem {
    fn default() -> Self {
        Self(None)
    }
}

impl CarriedItem {
    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn set(&mut self, item: Option<InventoryItem>) {
        self.0 = item;
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SlotArea {
    Hotbar,
    Backpack,
}

impl CarriedItem {
    pub(super) fn item(&self) -> Option<InventoryItem> {
        self.0
    }
}
