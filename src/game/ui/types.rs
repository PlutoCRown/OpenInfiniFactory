use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::game::state::UiPanelId;
use crate::game::state::{BuilderMode, GameMode, WorldEntryMode};
use crate::game::world::blocks::{edit_blocks, BlockKind, MaterialKind, StampColor, PLAY_BLOCKS};
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
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
pub struct PanelWindow;

#[derive(Component, Default)]
pub struct PanelPosition {
    pub dragged: bool,
    pub centered: bool,
}

#[derive(Component)]
pub struct PanelTitleBar;

#[derive(Component)]
pub struct PanelCloseButton;

#[derive(Resource, Default)]
pub struct PanelDragState {
    pub panel: Option<Entity>,
    pub cursor: Vec2,
    pub panel_pos: Vec2,
}

impl PanelDragState {
    pub fn clear(&mut self) {
        self.panel = None;
        self.cursor = Vec2::ZERO;
        self.panel_pos = Vec2::ZERO;
    }
}

#[derive(Resource, Default)]
pub struct UiHoverState {
    pub entity: Option<Entity>,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum PanelVisibility {
    GameMode(GameMode),
    SettingsTab(SettingsTab),
    ConfirmDialog,
    ModalScrim,
}

#[derive(Component)]
pub struct InGameHudStyle;

#[derive(Component)]
pub struct InGameHudVisibility;

#[derive(Component)]
pub struct GameplayHudVisibility;

#[derive(Component)]
pub struct ConverterInputRow;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockPanelText(pub BlockPanelTextKind);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockPanelTextKind {
    GeneratorPeriod,
    TeleportName,
}

#[derive(Resource, Default)]
pub struct OpenBlockPanelDropdown(pub Option<BlockPanelDropdown>);

#[derive(Component, Clone, Copy)]
pub struct KeyBindingButton(pub ConfigAction);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct PanelText(pub PanelTextKind);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PanelTextKind {
    InventoryTitle,
    SaveListTitle,
    ConfirmTitle,
    ConfirmMessage,
}

#[derive(Component)]
pub struct Crosshair;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsText(pub SettingsTextKind);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsTextKind {
    KeyBinding,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsValueText(pub SettingsField);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsSliderFill(pub SettingsField);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsSliderKnob(pub SettingsField);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownLabel(pub SettingsDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownList(pub SettingsDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownRow(pub SettingsDropdown);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsField {
    Fov,
    UiScale,
    Gravity,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SettingsSliderTrigger {
    Live,
    Commit,
}

#[derive(Clone, Copy)]
pub struct SettingsSliderConfig {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub trigger: SettingsSliderTrigger,
}

#[derive(Clone, Copy)]
pub enum SettingsControl {
    Slider {
        field: SettingsField,
        config: SettingsSliderConfig,
    },
    Dropdown(SettingsDropdown),
}

#[derive(Clone, Copy)]
pub struct SettingsItem {
    pub label_key: &'static str,
    pub control: SettingsControl,
}

pub const GAMEPLAY_SETTINGS: &[SettingsItem] = &[
    SettingsItem {
        label_key: "settings.fov",
        control: SettingsControl::Slider {
            field: SettingsField::Fov,
            config: SettingsSliderConfig {
                min: 50.0,
                max: 110.0,
                step: 1.0,
                trigger: SettingsSliderTrigger::Live,
            },
        },
    },
    SettingsItem {
        label_key: "settings.ui_scale_label",
        control: SettingsControl::Slider {
            field: SettingsField::UiScale,
            config: SettingsSliderConfig {
                min: UI_SCALE_MIN,
                max: UI_SCALE_MAX,
                step: 0.1,
                trigger: SettingsSliderTrigger::Commit,
            },
        },
    },
    SettingsItem {
        label_key: "settings.gravity",
        control: SettingsControl::Slider {
            field: SettingsField::Gravity,
            config: SettingsSliderConfig {
                min: GRAVITY_SCALE_MIN,
                max: GRAVITY_SCALE_MAX,
                step: 0.1,
                trigger: SettingsSliderTrigger::Commit,
            },
        },
    },
    SettingsItem {
        label_key: "settings.language",
        control: SettingsControl::Dropdown(SettingsDropdown::Language),
    },
    SettingsItem {
        label_key: "settings.place_selection_mode",
        control: SettingsControl::Dropdown(SettingsDropdown::PlaceSelectionMode),
    },
    SettingsItem {
        label_key: "settings.delete_selection_mode",
        control: SettingsControl::Dropdown(SettingsDropdown::DeleteSelectionMode),
    },
];

impl SettingsField {
    pub fn slider(self) -> Option<SettingsSliderConfig> {
        GAMEPLAY_SETTINGS
            .iter()
            .find_map(|item| match item.control {
                SettingsControl::Slider { field, config } if field == self => Some(config),
                _ => None,
            })
    }

    pub fn percent(self, settings: &crate::game::state::GameSettings) -> f32 {
        let Some(slider) = self.slider() else {
            return 0.0;
        };
        ((self.value(settings) - slider.min) / (slider.max - slider.min) * 100.0).clamp(0.0, 100.0)
    }

    pub fn display(
        self,
        settings: &crate::game::state::GameSettings,
        i18n: &crate::shared::i18n::I18n,
    ) -> String {
        match self {
            Self::Fov => format!("FOV {:.0}", settings.fov_degrees),
            Self::UiScale => i18n.fmt(
                "settings.ui_scale",
                &[("scale", format!("{:.1}", settings.ui_scale))],
            ),
            Self::Gravity => i18n.fmt(
                "settings.gravity_value",
                &[("scale", format!("{:.1}", settings.gravity_scale))],
            ),
        }
    }

    pub fn apply_percent(
        self,
        percent: f32,
        settings: &mut crate::game::state::GameSettings,
        ui_scale: &mut UiScale,
        config: &mut crate::shared::config::GameConfig,
    ) {
        let Some(slider) = self.slider() else {
            return;
        };
        let raw = slider.min + percent.clamp(0.0, 1.0) * (slider.max - slider.min);
        let value = (raw / slider.step).round() * slider.step;
        self.apply_value(
            value.clamp(slider.min, slider.max),
            settings,
            ui_scale,
            config,
        );
    }

    fn value(self, settings: &crate::game::state::GameSettings) -> f32 {
        match self {
            Self::Fov => settings.fov_degrees,
            Self::UiScale => settings.ui_scale,
            Self::Gravity => settings.gravity_scale,
        }
    }

    fn apply_value(
        self,
        value: f32,
        settings: &mut crate::game::state::GameSettings,
        ui_scale: &mut UiScale,
        config: &mut crate::shared::config::GameConfig,
    ) {
        match self {
            Self::Fov => {
                settings.fov_degrees = value;
                config.fov_degrees = value;
            }
            Self::UiScale => {
                settings.ui_scale = value;
                ui_scale.0 = value;
                config.ui_scale = value;
            }
            Self::Gravity => {
                settings.gravity_scale = value;
                config.gravity_scale = value;
            }
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

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum MenuAction {
    EditPuzzle,
    Play,
    Quit,
    Resume,
    ToggleBuilderMode,
    SaveWorld,
    SaveAsNewPuzzle,
    ResetSolution,
    OpenSettings,
    BackToMainMenu,
}

impl UiActionLabel for MenuAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::EditPuzzle => "button.edit_puzzle",
            Self::Play => "button.start_playing",
            Self::Quit => "button.quit_game",
            Self::Resume => "button.resume",
            Self::ToggleBuilderMode => "button.toggle_builder_mode",
            Self::SaveWorld => "button.save_world",
            Self::SaveAsNewPuzzle => "button.save_as_new_puzzle",
            Self::ResetSolution => "button.reset_solution",
            Self::OpenSettings => "button.settings",
            Self::BackToMainMenu => "button.back_to_main_menu",
        }
    }
}

#[derive(Component, Clone)]
pub enum SaveListAction {
    NewPuzzle,
    NewSolution,
    LoadPuzzle(String),
    LoadSolution(String),
    RenamePuzzle(String),
    RenameSolution(String),
    DeletePuzzle(String),
    DeleteSolution(String),
    Back,
}

#[derive(Component)]
pub struct SaveListCloseButton;

#[derive(Component)]
pub struct SaveListPanel;

#[derive(Component, Clone, Copy)]
pub struct SaveListPuzzleColumn;

#[derive(Component, Clone, Copy)]
pub struct SaveListSolutionColumn;

#[derive(Component)]
pub struct SaveListPrompt;

#[derive(Resource, Default)]
pub struct SaveListRenderState {
    pub entry: Option<WorldEntryMode>,
    pub puzzle_keys: Vec<String>,
    pub solution_keys: Vec<String>,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum TextPromptAction {
    Confirm,
    Cancel,
}

#[derive(Component)]
pub struct TextPromptRoot;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum TextPromptText {
    Title,
    Value,
}

#[derive(Clone, Eq, PartialEq)]
pub enum TextPromptKind {
    NewPuzzle,
    NewSolution { puzzle: String },
    RenamePuzzle { name: String },
    RenameSolution { name: String },
    SaveAsNewPuzzle,
}

#[derive(Resource, Default)]
pub struct TextPromptState {
    pub kind: Option<TextPromptKind>,
    pub value: String,
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
    EditPuzzleWithSolutions { name: String },
}

#[derive(Resource, Default)]
pub struct ConfirmDialogState {
    pub kind: Option<ConfirmDialogKind>,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum SettingsAction {
    TabGameplay,
    TabKeyBindings,
    Field(SettingsField),
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
            Self::Field(_)
            | Self::SetPlaceSelectionMode(_)
            | Self::SetDeleteSelectionMode(_)
            | Self::SetLanguage(_)
            | Self::ToggleDropdown(_) => "",
        }
    }
}

#[derive(Resource, Default)]
pub struct ActiveSettingsSlider(pub Option<SettingsField>);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum BlockEditAction {
    PeriodDown,
    PeriodUp,
    ToggleMaterialDropdown,
    SetMaterial(MaterialKind),
    ToggleColorDropdown,
    SetColor(StampColor),
    ToggleInputDropdown,
    ToggleOutputDropdown,
    SetInput(MaterialKind),
    SetOutput(MaterialKind),
}

impl UiActionLabel for BlockEditAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::PeriodDown => "button.period_down",
            Self::PeriodUp => "button.period_up",
            Self::ToggleMaterialDropdown | Self::SetMaterial(_) => "button.material_next",
            Self::ToggleColorDropdown | Self::SetColor(_) => "button.next_color",
            Self::ToggleInputDropdown | Self::SetInput(_) => "button.input_material",
            Self::ToggleOutputDropdown | Self::SetOutput(_) => "button.output_material",
        }
    }
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum TeleportAction {
    TogglePairDropdown,
    SetPair(Option<IVec3>),
    Rename,
}

impl UiActionLabel for TeleportAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::TogglePairDropdown | Self::SetPair(_) => "button.teleport_pair",
            Self::Rename => "button.teleport_rename",
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

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockMaterialIconSlot(pub BlockPanelDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockMaterialIcon(pub MaterialKind);

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
pub(crate) struct InventoryTooltip;

#[derive(Component)]
pub(crate) struct CarriedItemPreview;

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
        let edit_blocks = edit_blocks();
        let blocks: &[BlockKind] = match mode {
            BuilderMode::Edit => &edit_blocks,
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
