use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::game::state::UiPanelId;
use crate::game::state::{BuilderMode, GameMode};
use crate::game::world::blocks::{BlockKind, MaterialKind, StampColor, EDIT_BLOCKS, PLAY_BLOCKS};
use crate::shared::config::{ConfigAction, ConfigSelectionMode};
use crate::shared::i18n::Language;

pub const HOTBAR_SLOTS: usize = 9;
pub(super) const BACKPACK_SLOTS: usize = 27;
pub type HotbarItems = [Option<InventoryItem>; HOTBAR_SLOTS];

const PLAY_HOTBAR_BLOCKS: [BlockKind; HOTBAR_SLOTS] = [
    BlockKind::Platform,
    BlockKind::Welder,
    BlockKind::Conveyor,
    BlockKind::Detector,
    BlockKind::Wire,
    BlockKind::Pusher,
    BlockKind::Lifter,
    BlockKind::Rotator,
    BlockKind::Drill,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPanelContext {
    ReturnTo(GameMode),
    Block { pos: IVec3 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelSession {
    pub panel: UiPanelId,
    pub context: UiPanelContext,
}

#[derive(Resource, Default)]
pub struct UiRuntime {
    stack: Vec<UiPanelSession>,
}

impl UiRuntime {
    pub fn open(&mut self, panel: UiPanelId, context: UiPanelContext) {
        self.stack.retain(|session| session.panel != panel);
        self.stack.push(UiPanelSession { panel, context });
    }

    pub fn open_block(&mut self, panel: UiPanelId, pos: IVec3) {
        self.open(panel, UiPanelContext::Block { pos })
    }

    pub fn close_active(&mut self) {
        self.stack.pop();
    }

    pub fn close_current(&mut self) {
        self.close_active();
    }

    pub fn active(&self) -> Option<UiPanelSession> {
        self.stack.last().copied()
    }

    pub fn active_panel(&self) -> Option<UiPanelId> {
        self.active().map(|session| session.panel)
    }

    pub fn is_settings_open(&self) -> bool {
        self.active_panel().is_some_and(UiPanelId::is_settings)
    }

    pub fn blocks_gameplay(&self) -> bool {
        self.active_panel()
            .is_some_and(UiPanelId::is_blocking_gameplay)
    }

    pub fn active_block_pos(&self) -> Option<IVec3> {
        match self.active()?.context {
            UiPanelContext::Block { pos } => Some(pos),
            _ => None,
        }
    }

    pub fn panel_layer(&self, panel: UiPanelId) -> Option<usize> {
        self.stack.iter().position(|session| session.panel == panel)
    }

    pub fn top_modal_layer(&self) -> Option<usize> {
        self.stack
            .iter()
            .rposition(|session| session.panel.is_blocking_gameplay())
    }

    pub fn has_modal_panel(&self) -> bool {
        self.top_modal_layer().is_some()
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelBinding(pub UiPanelId);

#[derive(Component)]
pub struct ModalScrim;

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
pub struct GoalPanel;

#[derive(Component)]
pub struct LabelerPanel;

#[derive(Component)]
pub struct ConverterPanel;

#[derive(Component)]
pub struct ConverterInputRow;

#[derive(Component)]
pub struct TeleportPanel;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockPanelText(pub BlockPanelTextKind);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockPanelTextKind {
    GeneratorPeriod,
    TeleportName,
}

#[derive(Resource, Default)]
pub struct OpenBlockPanelDropdown(pub Option<BlockPanelDropdown>);

#[derive(Component)]
pub struct SettingsGameplayGroup;

#[derive(Component)]
pub struct SettingsKeyBindingsGroup;

#[derive(Component, Clone, Copy)]
pub struct KeyBindingButton(pub ConfigAction);

#[derive(Component)]
pub struct MainMenuPanel;

#[derive(Component)]
pub struct SaveListPanel;

#[derive(Component)]
pub struct SaveListTitle;

#[derive(Component)]
pub struct Crosshair;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsText(pub SettingsTextKind);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsTextKind {
    KeyBinding,
}

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

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownRoot(pub SettingsDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownRow(pub SettingsDropdown);

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
pub enum SettingsSliderUpdateMode {
    Live,
    Commit,
}

impl SettingsSlider {
    pub fn update_mode(self) -> SettingsSliderUpdateMode {
        match self {
            Self::Fov => SettingsSliderUpdateMode::Live,
            Self::UiScale | Self::Gravity => SettingsSliderUpdateMode::Commit,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsDropdown {
    Language,
    PlaceSelectionMode,
    DeleteSelectionMode,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct StatusText(pub StatusTextKind);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum StatusTextKind {
    Hotbar,
    CurrentSave,
    Simulation,
    SimulationOverlay,
}

#[derive(Component)]
pub struct LocalizedText {
    pub key: &'static str,
}

pub trait UiActionLabel {
    fn label_key(self) -> &'static str;
}

#[derive(Component, Clone, Copy)]
pub enum PauseAction {
    Resume,
    ToggleBuilderMode,
    SaveWorld,
    ResetSolution,
    OpenSettings,
    BackToMainMenu,
}

impl UiActionLabel for PauseAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::Resume => "button.resume",
            Self::ToggleBuilderMode => "button.toggle_builder_mode",
            Self::SaveWorld => "button.save_world",
            Self::ResetSolution => "button.reset_solution",
            Self::OpenSettings => "button.settings",
            Self::BackToMainMenu => "button.back_to_main_menu",
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum MainMenuAction {
    EditPuzzle,
    Play,
    OpenSettings,
    Quit,
}

impl UiActionLabel for MainMenuAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::EditPuzzle => "button.edit_puzzle",
            Self::Play => "button.start_playing",
            Self::OpenSettings => "button.settings",
            Self::Quit => "button.quit_game",
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum SaveListAction {
    NewPuzzle,
    NewSolution,
    LoadPuzzle(usize),
    LoadSolution(usize),
    DeletePuzzle(usize),
    DeleteSolution(usize),
    Back,
}

#[derive(Component, Clone, Copy)]
pub enum ConfirmDialogAction {
    Primary,
    Secondary,
    Cancel,
}

impl UiActionLabel for ConfirmDialogAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::Primary => "button.confirm",
            Self::Secondary => "button.confirm",
            Self::Cancel => "button.cancel",
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum ConfirmDialogKind {
    DeleteSave { name: String },
    ResetSolution,
    ReturnToMain,
    SaveSolutionBeforeEdit,
}

#[derive(Resource, Default)]
pub struct ConfirmDialogState {
    pub kind: Option<ConfirmDialogKind>,
}

#[derive(Component)]
pub struct ConfirmDialogPanel;

#[derive(Component)]
pub struct ConfirmDialogTitle;

#[derive(Component)]
pub struct ConfirmDialogMessage;

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

impl UiActionLabel for SettingsAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::TabGameplay => "button.gameplay",
            Self::TabKeyBindings => "button.key_bindings",
            Self::Bind(action) => action.label_key(),
            Self::ResetDefaults => "button.reset_defaults",
            Self::OpenFolder => "button.open_config_folder",
            Self::Back => "button.back",
            Self::FovSlider
            | Self::UiScaleSlider
            | Self::GravitySlider
            | Self::SetPlaceSelectionMode(_)
            | Self::SetDeleteSelectionMode(_)
            | Self::SetLanguage(_)
            | Self::ToggleDropdown(_) => "",
        }
    }
}

#[derive(Resource, Default)]
pub struct ActiveSettingsSlider(pub Option<SettingsSlider>);

#[derive(Component, Clone, Copy)]
pub enum GeneratorAction {
    PeriodDown,
    PeriodUp,
    ToggleMaterialDropdown,
    SetMaterial(MaterialKind),
    Close,
}

impl UiActionLabel for GeneratorAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::PeriodDown => "button.period_down",
            Self::PeriodUp => "button.period_up",
            Self::ToggleMaterialDropdown | Self::SetMaterial(_) => "button.material_next",
            Self::Close => "button.close",
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum LabelerAction {
    ToggleColorDropdown,
    SetColor(StampColor),
    Close,
}

impl UiActionLabel for LabelerAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleColorDropdown | Self::SetColor(_) => "button.next_color",
            Self::Close => "button.close",
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum ConverterAction {
    ToggleInputDropdown,
    ToggleOutputDropdown,
    SetInput(MaterialKind),
    SetOutput(MaterialKind),
    Close,
}

impl UiActionLabel for ConverterAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleInputDropdown | Self::SetInput(_) => "button.input_material",
            Self::ToggleOutputDropdown | Self::SetOutput(_) => "button.output_material",
            Self::Close => "button.close",
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum TeleportAction {
    TogglePairDropdown,
    SetPair(Option<IVec3>),
    Rename,
    Close,
}

impl UiActionLabel for TeleportAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::TogglePairDropdown | Self::SetPair(_) => "button.teleport_pair",
            Self::Rename => "button.teleport_rename",
            Self::Close => "button.close",
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum GoalAction {
    ToggleMaterialDropdown,
    SetMaterial(MaterialKind),
    Close,
}

impl UiActionLabel for GoalAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleMaterialDropdown | Self::SetMaterial(_) => "button.material_next",
            Self::Close => "button.close",
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockPanelDropdown {
    GeneratorMaterial,
    GoalMaterial,
    LabelerColor,
    ConverterInput,
    ConverterOutput,
    TeleportPair,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockPanelDropdownLabel(pub BlockPanelDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockPanelDropdownList(pub BlockPanelDropdown);

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
pub(crate) struct InventoryTooltip;

#[derive(Component)]
pub(crate) struct CarriedLabel;

#[derive(Component)]
pub(crate) struct CarriedIcon;

#[derive(Component, Clone, Copy)]
pub(crate) struct InventorySlot {
    pub area: SlotArea,
    pub index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

        let hotbar_blocks: &[BlockKind] = match mode {
            BuilderMode::Edit => blocks,
            BuilderMode::Play => &PLAY_HOTBAR_BLOCKS,
        };
        let mut hotbar = [None; HOTBAR_SLOTS];
        for (index, kind) in hotbar_blocks.iter().take(HOTBAR_SLOTS).enumerate() {
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

    pub fn set_hotbar(&mut self, hotbar: HotbarItems) {
        self.hotbar = hotbar;
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
