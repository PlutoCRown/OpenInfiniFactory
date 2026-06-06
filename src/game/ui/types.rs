//! Shared UI types and re-exports from modular subsystems.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::game::block_editing::OpenBlockPanelDropdown;
pub use crate::game::ui::core::{
    ConfirmButtonId, InlineTextEditState, PanelCloseButton, PanelDragState, PanelPosition,
    PanelTitleBar, PanelVisibility, PanelWindow, TextPromptRoot, TextPromptState, UiActionLabel,
    UiHost, UiHoverState, UiPanelBinding, UiRuntime,
};
pub use crate::game::ui::features::menu::types::MenuAction;
pub use crate::game::ui::features::save::types::{
    SaveListAction, SaveListCloseButton, SaveListPanel, SaveListPrompt, SaveListPuzzleColumn,
    SaveListRenderState, SaveListSolutionColumn, SaveListTitleText,
};
pub use crate::game::ui::features::settings::types::{
    OpenSettingsDropdown, PendingKeyBind, SettingsAction, SettingsControl, SettingsDropdown,
    SettingsDropdownLabel, SettingsDropdownList, SettingsDropdownRow, SettingsField, SettingsItem,
    SettingsSliderFill, SettingsSliderKnob, SettingsTab, SettingsText, SettingsTextKind,
    SettingsValueText, GAMEPLAY_SETTINGS,
};

use crate::game::state::BuilderMode;
use crate::game::blocks::{edit_blocks, BlockKind, PLAY_BLOCKS};
use crate::shared::config::ActionKeyName;

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

#[derive(Component)]
pub struct UiRoot;

#[derive(Component)]
pub struct PlayingUiRoot;

#[derive(Component)]
pub struct InGameHudStyle;

#[derive(Component)]
pub struct InGameHudVisibility;

#[derive(Component)]
pub struct GameplayHudVisibility;

#[derive(Component)]
pub struct ConverterInputRow;

#[derive(Component, Clone, Copy)]
pub struct KeyBindingButton(pub ActionKeyName);

#[derive(Component)]
pub struct Crosshair;

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

#[derive(Component)]
pub(crate) struct InventoryTooltip;

#[derive(Component)]
pub(crate) struct CarriedItemPreview;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct InventorySlot {
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

    pub(super) fn item(&self) -> Option<InventoryItem> {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotArea {
    Hotbar,
    Backpack,
}
