use bevy::prelude::*;
use std::collections::VecDeque;

use crate::game::state::{BuilderMode, GameMode};
pub use crate::game::state::UiPanelId;
use crate::game::world::blocks::{BlockKind, EDIT_BLOCKS, PLAY_BLOCKS};
use crate::shared::config::{ConfigAction, ConfigSelectionMode};
use crate::shared::i18n::Language;

pub const HOTBAR_SLOTS: usize = 9;
pub(super) const BACKPACK_SLOTS: usize = 27;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiRequestId(u64);

impl UiRequestId {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPanelContext {
    None,
    ReturnTo(GameMode),
    Block { pos: IVec3 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPanelResult {
    Closed,
    SettingsClosed,
    BlockClosed { pos: IVec3 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelSession {
    pub request_id: UiRequestId,
    pub panel: UiPanelId,
    pub context: UiPanelContext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelResponse {
    pub request_id: UiRequestId,
    pub panel: UiPanelId,
    pub context: UiPanelContext,
    pub result: UiPanelResult,
}

#[derive(Resource, Default)]
pub struct UiRuntime {
    active: Option<UiPanelSession>,
    next_request_id: u64,
    results: VecDeque<UiPanelResponse>,
}

impl UiRuntime {
    pub fn open(&mut self, panel: UiPanelId, context: UiPanelContext) -> UiRequestId {
        self.close_active(UiPanelResult::Closed);
        let request_id = UiRequestId(self.next_request_id);
        self.next_request_id = self.next_request_id.wrapping_add(1);
        self.active = Some(UiPanelSession {
            request_id,
            panel,
            context,
        });
        request_id
    }

    pub fn open_block(&mut self, panel: UiPanelId, pos: IVec3) -> UiRequestId {
        self.open(panel, UiPanelContext::Block { pos })
    }

    pub fn close_active(&mut self, result: UiPanelResult) -> Option<UiPanelResponse> {
        let session = self.active.take()?;
        let response = UiPanelResponse {
            request_id: session.request_id,
            panel: session.panel,
            context: session.context,
            result,
        };
        self.results.push_back(response);
        Some(response)
    }

    pub fn close_current(&mut self) -> Option<UiPanelResponse> {
        let result = self
            .active_block_pos()
            .map(|pos| UiPanelResult::BlockClosed { pos })
            .unwrap_or(UiPanelResult::Closed);
        self.close_active(result)
    }

    pub fn active(&self) -> Option<UiPanelSession> {
        self.active
    }

    pub fn active_panel(&self) -> Option<UiPanelId> {
        self.active.map(|session| session.panel)
    }

    pub fn is_settings_open(&self) -> bool {
        self.active_panel().is_some_and(UiPanelId::is_settings)
    }

    pub fn blocks_gameplay(&self) -> bool {
        self.active_panel()
            .is_some_and(UiPanelId::is_blocking_gameplay)
    }

    pub fn active_block_pos(&self) -> Option<IVec3> {
        match self.active?.context {
            UiPanelContext::Block { pos } => Some(pos),
            _ => None,
        }
    }

    pub fn take_result(&mut self, request_id: UiRequestId) -> Option<UiPanelResponse> {
        let index = self
            .results
            .iter()
            .position(|response| response.request_id == request_id)?;
        self.results.remove(index)
    }

    pub fn drain_results(&mut self) -> Vec<UiPanelResponse> {
        self.results.drain(..).collect()
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelBinding(pub UiPanelId);

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
pub struct ConverterPanel;

#[derive(Component)]
pub struct ConverterInputRow;

#[derive(Component)]
pub struct ConverterModeText;

#[derive(Component)]
pub struct ConverterInputText;

#[derive(Component)]
pub struct ConverterOutputText;

#[derive(Component)]
pub struct TeleportPanel;

#[derive(Component)]
pub struct TeleportNameText;

#[derive(Component)]
pub struct TeleportPairText;

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
    Gravity,
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
    Gravity,
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
pub struct SimulationStatusText;

#[derive(Component)]
pub struct LocalizedText {
    pub key: &'static str,
}

#[derive(Component, Clone, Copy)]
pub enum PauseAction {
    Resume,
    ToggleBuilderMode,
    ConfirmSaveSolutionAndEdit,
    DiscardSolutionAndEdit,
    CancelEditSwitch,
    SaveAndBackToMain,
    DiscardAndBackToMain,
    CancelBackToMain,
    SaveWorld,
    ResetSolution,
    ConfirmResetSolution,
    CancelResetSolution,
    OpenSettings,
    BackToMainMenu,
}

#[derive(Component, Clone, Copy)]
pub enum MainMenuAction {
    EditPuzzle,
    Play,
    OpenSettings,
    Quit,
}

#[derive(Component, Clone, Copy)]
pub enum SaveListAction {
    NewPuzzle,
    NewSolution,
    LoadPuzzle(usize),
    LoadSolution(usize),
    DeletePuzzle(usize),
    DeleteSolution(usize),
    ConfirmDelete,
    CancelDelete,
    Back,
}

#[derive(Component, Clone, Copy)]
pub enum SettingsAction {
    TabGameplay,
    TabKeyBindings,
    FovSlider,
    UiScaleSlider,
    GravitySlider,
    SetPlaceSelectionMode(ConfigSelectionMode),
    SetDeleteSelectionMode(ConfigSelectionMode),
    SetLanguage(Language),
    ToggleDropdown(SettingsDropdown),
    Bind(ConfigAction),
    ResetDefaults,
    OpenFolder,
    Back,
}

#[derive(Resource, Default)]
pub struct ActiveSettingsSlider(pub Option<SettingsSlider>);

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

#[derive(Component, Clone, Copy)]
pub enum ConverterAction {
    ToggleMode,
    InputNext,
    OutputNext,
    Close,
}

#[derive(Component, Clone, Copy)]
pub enum TeleportAction {
    CyclePair,
    Rename,
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
pub struct PendingAppExit {
    pub requested: bool,
    pub exit: Option<AppExit>,
}

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
        if mode == BuilderMode::Edit {
            if let Some(slot) = backpack.iter_mut().find(|slot| slot.is_none()) {
                *slot = Some(InventoryItem::Area(AreaKind::Selection));
            }
        }

        Self { hotbar, backpack }
    }

    pub fn can_take_block(&self, kind: BlockKind) -> bool {
        let item = Some(InventoryItem::Block(kind));
        self.hotbar.contains(&item) || self.backpack.contains(&item)
    }

    pub fn hotbar_index_of_block(&self, kind: BlockKind) -> Option<usize> {
        let item = Some(InventoryItem::Block(kind));
        self.hotbar.iter().position(|candidate| *candidate == item)
    }

    pub fn set_hotbar_block(&mut self, index: usize, kind: BlockKind) {
        if let Some(slot) = self.hotbar.get_mut(index) {
            *slot = Some(InventoryItem::Block(kind));
        }
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

    pub fn take(&mut self) -> Option<InventoryItem> {
        self.0.take()
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
