use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui_widgets::{CoreSliderDragState, Slider, SliderRange, SliderValue};
use bevy::window::PrimaryWindow;

use crate::game::state::{
    BuilderMode, GameMode, GameSettings, PlacementState, SimulationState, SolutionState,
    TeleportRenameState, WorldEntryMode,
};
use crate::game::world::blocks::BlockKind;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;
use crate::shared::config::{ConfigAction, GameConfig};
use crate::shared::i18n::{I18n, Language};
use crate::shared::save::{SaveKind, SaveState};

use super::components::{
    hover_border, inset_border, menu_button, pressed_border, raised_border, BUTTON_BG,
    BUTTON_HOVER_BG, BUTTON_PRESSED_BG,
};
use super::types::{
    ActiveSettingsSlider, BlockPanelDropdown, BlockPanelDropdownLabel, BlockPanelDropdownList,
    BlockPanelText, BlockPanelTextKind, CarriedItem, CarriedItemPreview, ConfirmDialogAction,
    ConfirmDialogKind, ConfirmDialogState, ConverterInputRow, Crosshair, InGameHudStyle,
    InGameHudVisibility, InventoryItems, InventorySlot, InventoryTooltip, KeyBindingButton,
    LocalizedText, OpenBlockPanelDropdown, OpenSettingsDropdown, PanelText, PanelTextKind,
    MenuAction, PanelVisibility, PendingKeyBind, SaveListAction, ScrollContainer, ScrollContent,
    SettingsAction, SettingsDropdownLabel, SettingsDropdownList, SettingsDropdownRoot,
    SettingsDropdownRow, SettingsField, SettingsSliderFill, SettingsSliderKnob, SettingsTab,
    SettingsText, SettingsTextKind, SettingsValueText, SlotArea, StatusText, StatusTextKind,
    TeleportAction, UiPanelBinding, UiPanelId, UiRuntime,
};
use super::widgets::{short_item_name, slot_color};
include!("font.rs");
include!("inventory_actions.rs");
include!("status.rs");
include!("cursor_scroll.rs");
include!("block_panels.rs");
include!("settings.rs");
include!("localized.rs");
include!("panels.rs");
include!("hud.rs");
include!("inventory_render.rs");
include!("save_dialogs.rs");
