use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::game::state::UiPanelId;
use crate::game::state::{BuilderMode, GameMode, WorldEntryMode};
use crate::game::world::blocks::{edit_blocks, BlockKind, MaterialKind, StampColor, PLAY_BLOCKS};
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{ConfigAction, ConfigSelectionMode};
use crate::shared::i18n::{I18n, Language};

pub const HOTBAR_SLOTS: usize = 9;
pub(super) const BACKPACK_SLOTS: usize = 27;
pub type HotbarItems = [Option<InventoryItem>; HOTBAR_SLOTS];

const PLAY_HOTBAR_BLOCKS: [BlockKind; HOTBAR_SLOTS] = [
    BlockKind::Platform,
    BlockKind::Welder,
    BlockKind::Conveyor,
    BlockKind::Detector,
    BlockKind::Wire,
    BlockKind::Switch,
    BlockKind::Pusher,
    BlockKind::Lifter,
    BlockKind::Rotator,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPanelContext {
    None,
    ReturnTo(GameMode),
    Block { pos: IVec3 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiPanelKey(pub &'static str);

impl UiPanelKey {
    pub const MAIN_MENU: Self = Self("core.main_menu");
    pub const SAVE_LIST: Self = Self("core.save_list");
    pub const PAUSE_MENU: Self = Self("core.pause_menu");
    pub const INVENTORY: Self = Self("core.inventory");
    pub const SETTINGS: Self = Self("core.settings");
    pub const GENERATOR: Self = Self("block.generator");
    pub const GOAL: Self = Self("block.goal");
    pub const LABELER: Self = Self("block.labeler");
    pub const CONVERTER: Self = Self("block.converter");
    pub const TELEPORT: Self = Self("block.teleport");

    pub fn is_settings(self) -> bool {
        self == Self::SETTINGS
    }
}

#[derive(Clone, Copy)]
pub struct UiPanelDescriptor {
    pub key: UiPanelKey,
    pub title_key: &'static str,
    pub blocks_gameplay: bool,
    pub spawn: fn(&mut ChildSpawnerCommands, &I18n) -> Entity,
}

impl UiPanelDescriptor {
    pub const fn new(
        key: UiPanelKey,
        title_key: &'static str,
        blocks_gameplay: bool,
        spawn: fn(&mut ChildSpawnerCommands, &I18n) -> Entity,
    ) -> Self {
        Self {
            key,
            title_key,
            blocks_gameplay,
            spawn,
        }
    }
}

#[derive(Resource, Default)]
pub struct UiPanelRegistry {
    panels: HashMap<UiPanelKey, UiPanelDescriptor>,
}

impl UiPanelRegistry {
    pub fn register(&mut self, descriptor: UiPanelDescriptor) {
        let _ = (descriptor.title_key, descriptor.blocks_gameplay);
        self.panels.insert(descriptor.key, descriptor);
    }

    pub fn get(&self, key: UiPanelKey) -> Option<&UiPanelDescriptor> {
        self.panels.get(&key)
    }
}

#[derive(Component)]
pub struct UiRoot;

#[derive(Component)]
pub struct InventoryRuntimeEntity;

#[derive(Resource, Default)]
pub struct UiPanelHost {
    spawned: HashMap<UiPanelKey, Entity>,
}

impl UiPanelHost {
    pub fn get(&self, key: UiPanelKey) -> Option<Entity> {
        self.spawned.get(&key).copied()
    }

    pub fn insert(&mut self, key: UiPanelKey, entity: Entity) {
        self.spawned.insert(key, entity);
    }
}

#[derive(Message, Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenUiPanel {
    pub key: UiPanelKey,
    pub context: UiPanelContext,
}

impl OpenUiPanel {
    pub const fn new(key: UiPanelKey, context: UiPanelContext) -> Self {
        Self { key, context }
    }
}

#[derive(Message, Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloseUiPanel {
    pub key: Option<UiPanelKey>,
}

#[derive(Message, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelOpened {
    pub key: UiPanelKey,
    pub context: UiPanelContext,
}

#[derive(Message, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelClosed {
    pub key: UiPanelKey,
}

#[derive(Message, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelContextChanged {
    pub key: UiPanelKey,
    pub old: UiPanelContext,
    pub new: UiPanelContext,
}

#[derive(Message, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SettingsChanged;

#[derive(Message, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InventoryChanged;

#[derive(Message, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LanguageChanged;

#[derive(Message, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SaveListChanged;

#[derive(Message, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GameplayUiChanged;

#[derive(Message, Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockSettingsChanged {
    pub pos: IVec3,
}

#[derive(Message, Clone)]
pub struct OpenConfirmDialog(pub ConfirmDialogSpec);

#[derive(Message, Clone)]
pub struct OpenTextPrompt {
    pub kind: TextPromptKind,
    pub value: String,
}

impl OpenTextPrompt {
    pub fn new(kind: TextPromptKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }
}

#[derive(Message, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CloseUiModal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiModalKind {
    ConfirmDialog,
    TextPrompt,
}

#[derive(Message, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiModalOpened {
    pub kind: UiModalKind,
}

#[derive(Message, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiModalClosed {
    pub kind: UiModalKind,
}

#[derive(Component, Clone, Copy)]
pub struct OpensPanel {
    pub key: UiPanelKey,
    pub context: UiPanelContext,
}

#[derive(Component, Clone, Copy)]
pub struct ConfirmDialogRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelSession {
    pub panel: UiPanelKey,
    pub context: UiPanelContext,
    pub blocks_gameplay: bool,
}

#[derive(Resource, Default)]
pub struct UiRuntime {
    stack: Vec<UiPanelSession>,
    modal: Option<UiModal>,
}

impl UiRuntime {
    pub fn open_key(
        &mut self,
        panel: UiPanelKey,
        context: UiPanelContext,
        blocks_gameplay: bool,
    ) -> Option<UiPanelContext> {
        let previous = self
            .stack
            .iter()
            .find(|session| session.panel == panel)
            .map(|session| session.context);
        self.stack.retain(|session| session.panel != panel);
        self.stack.push(UiPanelSession {
            panel,
            context,
            blocks_gameplay,
        });
        previous
    }

    pub fn close_active(&mut self) -> Option<UiPanelSession> {
        self.stack.pop()
    }

    pub fn close_panel(&mut self, panel: UiPanelKey) -> Option<UiPanelSession> {
        if let Some(index) = self
            .stack
            .iter()
            .rposition(|session| session.panel == panel)
        {
            Some(self.stack.remove(index))
        } else {
            None
        }
    }

    pub fn active(&self) -> Option<UiPanelSession> {
        self.stack.last().copied()
    }

    pub fn active_key(&self) -> Option<UiPanelKey> {
        self.active().map(|session| session.panel)
    }

    pub fn is_settings_open(&self) -> bool {
        self.active_key().is_some_and(UiPanelKey::is_settings)
    }

    pub fn blocks_gameplay(&self) -> bool {
        self.active().is_some_and(|session| session.blocks_gameplay)
    }

    pub fn active_block_pos(&self) -> Option<IVec3> {
        match self.active()?.context {
            UiPanelContext::Block { pos } => Some(pos),
            _ => None,
        }
    }

    pub fn panel_layer(&self, panel: UiPanelKey) -> Option<usize> {
        self.stack.iter().position(|session| session.panel == panel)
    }

    pub fn top_modal_layer(&self) -> Option<usize> {
        self.stack
            .iter()
            .rposition(|session| session.blocks_gameplay)
    }

    pub fn has_modal_panel(&self) -> bool {
        self.top_modal_layer().is_some()
    }

    pub fn open_confirm_dialog(&mut self, spec: ConfirmDialogSpec) {
        self.modal = Some(UiModal::Confirm(ConfirmDialogState::from(spec)));
    }

    pub fn open_text_prompt(&mut self, kind: TextPromptKind, value: &str) {
        self.modal = Some(UiModal::TextPrompt(TextPromptState {
            kind,
            value: value.chars().take(24).collect(),
        }));
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    pub fn modal_kind(&self) -> Option<UiModalKind> {
        match self.modal.as_ref()? {
            UiModal::Confirm(_) => Some(UiModalKind::ConfirmDialog),
            UiModal::TextPrompt(_) => Some(UiModalKind::TextPrompt),
        }
    }

    pub fn confirm_dialog(&self) -> Option<&ConfirmDialogState> {
        match self.modal.as_ref()? {
            UiModal::Confirm(dialog) => Some(dialog),
            _ => None,
        }
    }

    pub fn text_prompt(&self) -> Option<&TextPromptState> {
        match self.modal.as_ref()? {
            UiModal::TextPrompt(prompt) => Some(prompt),
            _ => None,
        }
    }

    pub fn text_prompt_mut(&mut self) -> Option<&mut TextPromptState> {
        match self.modal.as_mut()? {
            UiModal::TextPrompt(prompt) => Some(prompt),
            _ => None,
        }
    }

    pub fn has_modal(&self) -> bool {
        self.modal.is_some()
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelBinding(pub UiPanelKey);

impl From<UiPanelId> for UiPanelBinding {
    fn from(panel: UiPanelId) -> Self {
        Self(match panel {
            UiPanelId::Settings => UiPanelKey::SETTINGS,
            UiPanelId::Generator => UiPanelKey::GENERATOR,
            UiPanelId::Goal => UiPanelKey::GOAL,
            UiPanelId::Labeler => UiPanelKey::LABELER,
            UiPanelId::Converter => UiPanelKey::CONVERTER,
            UiPanelId::Teleport => UiPanelKey::TELEPORT,
        })
    }
}

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

#[derive(Component, Clone, Copy, Default, Eq, PartialEq)]
pub struct BlockPanelText {
    pub kind: BlockPanelTextKind,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub enum BlockPanelTextKind {
    #[default]
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
pub struct SettingsDropdownLabel(pub SettingsDropdownId);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownList(pub SettingsDropdownId);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct SettingsDropdownRow(pub SettingsDropdownId);

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
    Dropdown(SettingsDropdownSpec),
}

#[derive(Clone, Copy)]
pub struct SettingsItem {
    pub text_key: &'static str,
    pub control: SettingsControl,
}

pub const GAMEPLAY_SETTINGS: &[SettingsItem] = &[
    SettingsItem {
        text_key: "settings.fov",
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
        text_key: "settings.ui_scale_label",
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
        text_key: "settings.gravity",
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
        text_key: "settings.language",
        control: SettingsControl::Dropdown(SettingsDropdownSpec::LANGUAGE),
    },
    SettingsItem {
        text_key: "settings.place_selection_mode",
        control: SettingsControl::Dropdown(SettingsDropdownSpec::PLACE_SELECTION_MODE),
    },
    SettingsItem {
        text_key: "settings.delete_selection_mode",
        control: SettingsControl::Dropdown(SettingsDropdownSpec::DELETE_SELECTION_MODE),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SettingsDropdownId(pub u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsDropdownValue {
    Language,
    PlaceSelectionMode,
    DeleteSelectionMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SettingsDropdownSpec {
    pub id: SettingsDropdownId,
    pub value: SettingsDropdownValue,
}

impl SettingsDropdownSpec {
    pub const LANGUAGE: Self = Self {
        id: SettingsDropdownId(0),
        value: SettingsDropdownValue::Language,
    };
    pub const PLACE_SELECTION_MODE: Self = Self {
        id: SettingsDropdownId(1),
        value: SettingsDropdownValue::PlaceSelectionMode,
    };
    pub const DELETE_SELECTION_MODE: Self = Self {
        id: SettingsDropdownId(2),
        value: SettingsDropdownValue::DeleteSelectionMode,
    };
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

#[derive(Component, Clone, Copy, Default)]
pub struct LocalizedText {
    pub key: &'static str,
}

#[derive(Clone, Copy)]
pub struct ButtonSpec<Action> {
    pub text: &'static str,
    pub on_click: Action,
}

impl<Action> ButtonSpec<Action> {
    pub const fn new(text: &'static str, on_click: Action) -> Self {
        Self { text, on_click }
    }
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum MainMenuAction {
    EditPuzzle,
    Play,
    Quit,
    OpenSettings,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum PauseMenuAction {
    Resume,
    ToggleBuilderMode,
    SaveWorld,
    SaveAsNewPuzzle,
    ResetSolution,
    OpenSettings,
    BackToMainMenu,
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

#[derive(Clone, Eq, PartialEq)]
pub struct TextPromptState {
    pub kind: TextPromptKind,
    pub value: String,
}

#[derive(Component, Clone, Copy)]
pub enum ConfirmDialogAction {
    Primary,
    Secondary,
    Cancel,
}

impl ConfirmDialogAction {
    pub fn result(self) -> ConfirmDialogResult {
        match self {
            Self::Primary => ConfirmDialogResult::Primary,
            Self::Secondary => ConfirmDialogResult::Secondary,
            Self::Cancel => ConfirmDialogResult::Cancel,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ConfirmDialogResult {
    Primary,
    Secondary,
    Cancel,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ConfirmDialogButtonSpec {
    pub text_key: &'static str,
    pub effect: ConfirmDialogEffect,
}

impl ConfirmDialogButtonSpec {
    pub fn new(text_key: &'static str, effect: ConfirmDialogEffect) -> Self {
        Self { text_key, effect }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ConfirmDialogSpec {
    pub title_key: &'static str,
    pub message: ConfirmDialogMessage,
    pub primary: ConfirmDialogButtonSpec,
    pub secondary: Option<ConfirmDialogButtonSpec>,
    pub cancel_key: &'static str,
}

impl ConfirmDialogSpec {
    pub fn new(
        message: ConfirmDialogMessage,
        primary: ConfirmDialogButtonSpec,
        secondary: Option<ConfirmDialogButtonSpec>,
    ) -> Self {
        Self {
            title_key: "confirm.title",
            message,
            primary,
            secondary,
            cancel_key: "button.cancel",
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ConfirmDialogState {
    pub title_key: &'static str,
    pub message: ConfirmDialogMessage,
    pub primary_key: &'static str,
    pub primary_effect: ConfirmDialogEffect,
    pub secondary_key: Option<&'static str>,
    pub secondary_effect: Option<ConfirmDialogEffect>,
    pub cancel_key: &'static str,
}

impl From<ConfirmDialogSpec> for ConfirmDialogState {
    fn from(spec: ConfirmDialogSpec) -> Self {
        let (secondary_key, secondary_effect) = spec
            .secondary
            .map(|button| (Some(button.text_key), Some(button.effect)))
            .unwrap_or((None, None));
        Self {
            title_key: spec.title_key,
            message: spec.message,
            primary_key: spec.primary.text_key,
            primary_effect: spec.primary.effect,
            secondary_key,
            secondary_effect,
            cancel_key: spec.cancel_key,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum ConfirmDialogMessage {
    TextKey(&'static str),
    Named { key: &'static str, name: String },
}

#[derive(Clone, Eq, PartialEq)]
pub enum ConfirmDialogEffect {
    None,
    DeleteSave { name: String },
    ResetSolution,
    ReturnToMain { save_first: bool },
    SwitchToEditMode { save_first: bool },
    OpenPuzzleForEdit { name: String },
    SaveCurrentWorld,
    SaveAsNewPuzzle { default_name: String },
}

#[derive(Clone, Eq, PartialEq)]
pub enum UiModal {
    Confirm(ConfirmDialogState),
    TextPrompt(TextPromptState),
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum SettingsAction {
    TabGameplay,
    TabKeyBindings,
    Field(SettingsField),
    SetPlaceSelectionMode(ConfigSelectionMode),
    SetDeleteSelectionMode(ConfigSelectionMode),
    SetLanguage(Language),
    ToggleDropdown(SettingsDropdownId),
    Bind(ConfigAction),
    ResetDefaults,
    OpenFolder,
    Back,
}

#[derive(Resource, Default)]
pub struct ActiveSettingsSlider(pub Option<SettingsField>);

#[derive(Component, Clone, Copy, Default, Eq, PartialEq)]
pub enum BlockEditAction {
    #[default]
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

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum TeleportAction {
    TogglePairDropdown,
    SetPair(Option<IVec3>),
    Rename,
}

#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlockPanelDropdown {
    #[default]
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

#[derive(Component, Clone, Copy, Default, Eq, PartialEq)]
pub struct BlockMaterialIconSlot {
    pub dropdown: BlockPanelDropdown,
}

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
pub struct OpenSettingsDropdown(pub Option<SettingsDropdownId>);

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
