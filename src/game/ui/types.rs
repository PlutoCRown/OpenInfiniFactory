//! Shared UI types and re-exports from modular subsystems.
use bevy::prelude::*;

pub use crate::game::block_editing::OpenBlockPanelDropdown;
pub use crate::game::ui::core::{
    ConfirmButtonId, InlineTextEditState, PanelCloseButton, PanelDragState, PanelFlowLayout,
    PanelPosition, PanelTitleBar, PanelVisibility, PanelWindow, TextPromptRoot, TextPromptState,
    UiActionLabel, UiHost, UiHoverState, UiPanelBinding, UiRuntime,
};
pub use crate::game::ui::features::save::types::{
    SaveListAction, SaveListCloseButton, SaveListCreateButton, SaveListPanel, SaveListPrompt,
    SaveListPuzzleColumn, SaveListPuzzleRows, SaveListRenderState, SaveListSolutionColumn,
    SaveListSolutionRows, SaveListTitleText,
};
pub use crate::game::ui::features::settings::types::{
    GAMEPLAY_SETTINGS, GRAPHICS_SETTINGS, OpenSettingsDropdown, PendingKeyBind, SettingsAction,
    SettingsControl, SettingsDropdown, SettingsDropdownLabel, SettingsDropdownList,
    SettingsDropdownRow, SettingsField, SettingsItem, SettingsSliderFill, SettingsSliderKnob,
    SettingsTab, SettingsText, SettingsTextKind, SettingsValueText,
};

use crate::game::blocks::{BlockKind, PLAY_BLOCKS, edit_blocks};
use crate::game::state::BuilderMode;
use crate::shared::config::ActionKeyName;
use crate::shared::save::{SavedAreaKind, SavedHotbar, SavedHotbarItem};

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

#[derive(Component, Clone, Copy)]
pub struct KeyBindingButton(pub ActionKeyName);

#[derive(Component)]
pub struct Crosshair;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct StatusText(pub StatusTextKind);

/// 状态栏文本种类
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum StatusTextKind {
    /// 左上角建造信息（世界 / 手持 / 瞄准）
    Gameplay,
    /// 右下角模拟状态（回合 / 播放状态 / 快捷键提示）
    SimulationOverlay,
}

#[derive(Component)]
pub struct LocalizedText {
    pub key: &'static str,
}

#[derive(Component)]
pub struct InventoryTooltip;

/// 背包悬停标签的名称文字
#[derive(Component)]
pub struct InventoryTooltipName;

/// 背包悬停标签的描述文字
#[derive(Component)]
pub struct InventoryTooltipDescription;

#[derive(Component)]
pub struct CarriedItemPreview;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct InventorySlot {
    pub area: SlotArea,
    pub index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AreaKind {
    Selection,
}

impl AreaKind {
    pub fn name_key(self) -> &'static str {
        match self {
            Self::Selection => "area.selection",
        }
    }

    pub fn description_key(self) -> &'static str {
        match self {
            Self::Selection => "desc.area.selection",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryItem {
    Block(BlockKind),
    Area(AreaKind),
    /// 灯面板：贴在电线表面，不占邻格
    LightPanel,
}

impl InventoryItem {
    pub fn name_key(self) -> &'static str {
        match self {
            Self::Block(kind) => kind.name_key(),
            Self::Area(kind) => kind.name_key(),
            Self::LightPanel => "item.light_panel",
        }
    }

    pub fn description_key(self) -> &'static str {
        match self {
            Self::Block(kind) => kind.description_key(),
            Self::Area(kind) => kind.description_key(),
            Self::LightPanel => "desc.item.light_panel",
        }
    }

    pub fn block(self) -> Option<BlockKind> {
        match self {
            Self::Block(kind) => Some(kind),
            Self::Area(_) | Self::LightPanel => None,
        }
    }

    pub fn area(self) -> Option<AreaKind> {
        match self {
            Self::Area(kind) => Some(kind),
            Self::Block(_) | Self::LightPanel => None,
        }
    }

    pub fn is_light_panel(self) -> bool {
        matches!(self, Self::LightPanel)
    }
}

#[derive(Resource)]
pub struct InventoryItems {
    pub hotbar: [Option<InventoryItem>; HOTBAR_SLOTS],
    pub(super) backpack: [Option<InventoryItem>; BACKPACK_SLOTS],
    /// 编辑→游玩时暂存快捷栏，切回编辑时恢复
    stashed_edit_hotbar: Option<HotbarItems>,
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
        } else {
            if let Some(slot) = backpack.iter_mut().find(|slot| slot.is_none()) {
                *slot = Some(InventoryItem::Area(AreaKind::Selection));
            }
            if let Some(slot) = backpack.iter_mut().find(|slot| slot.is_none()) {
                *slot = Some(InventoryItem::LightPanel);
            }
        }

        Self {
            hotbar,
            backpack,
            stashed_edit_hotbar: None,
        }
    }

    /// 编辑切游玩：暂存编辑快捷栏并换成游玩物品栏
    pub fn begin_play_from_edit(&mut self) {
        let edit_hotbar = self.hotbar;
        *self = Self::for_mode(BuilderMode::Play);
        self.stashed_edit_hotbar = Some(edit_hotbar);
    }

    /// 游玩切回编辑：恢复暂存的编辑快捷栏（无暂存则用默认）
    pub fn return_to_edit(&mut self) {
        let stash = self.stashed_edit_hotbar.take();
        *self = Self::for_mode(BuilderMode::Edit);
        if let Some(hotbar) = stash {
            self.hotbar = hotbar;
        }
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

    /// 导出存档用快捷栏 DTO
    pub fn to_saved_hotbar(&self) -> SavedHotbar {
        self.hotbar.map(|slot| {
            slot.map(|item| match item {
                InventoryItem::Block(kind) => SavedHotbarItem::Block(kind),
                InventoryItem::Area(AreaKind::Selection) => {
                    SavedHotbarItem::Area(SavedAreaKind::Selection)
                }
                InventoryItem::LightPanel => SavedHotbarItem::LightPanel,
            })
        })
    }

    /// 从存档快捷栏 DTO 写回 UI
    pub fn apply_saved_hotbar(&mut self, hotbar: SavedHotbar) {
        self.hotbar = hotbar.map(|slot| {
            slot.map(|item| match item {
                SavedHotbarItem::Block(kind) => InventoryItem::Block(kind),
                SavedHotbarItem::Area(SavedAreaKind::Selection) => {
                    InventoryItem::Area(AreaKind::Selection)
                }
                SavedHotbarItem::LightPanel => InventoryItem::LightPanel,
            })
        });
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

    pub fn item(&self) -> Option<InventoryItem> {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotArea {
    Hotbar,
    Backpack,
}
