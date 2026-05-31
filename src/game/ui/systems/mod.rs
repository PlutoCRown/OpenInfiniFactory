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
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{ConfigAction, GameConfig};
use crate::shared::i18n::{I18n, Language};
use crate::shared::save::{SaveKind, SaveState};

use super::components::{
    hover_border, inset_border, menu_button, pressed_border, raised_border, BUTTON_BG,
    BUTTON_HOVER_BG, BUTTON_PRESSED_BG,
};
use super::types::{
    ActiveSettingsSlider, BackpackPanel, BlockPanelDropdown, BlockPanelDropdownLabel,
    BlockPanelDropdownList, CarriedIcon, CarriedItem, CarriedLabel, ConfirmDialogAction,
    ConfirmDialogKind, ConfirmDialogMessage, ConfirmDialogPanel,
    ConfirmDialogState, ConfirmDialogTitle, ConverterInputRow,
    Crosshair,
    InGameHudStyle, InGameHudVisibility, InventoryItems, InventorySlot, InventoryTitle,
    InventoryTooltip, KeyBindingButton, LocalizedText,
    MainMenuPanel, ModalScrim, OpenBlockPanelDropdown, OpenSettingsDropdown, PauseAction,
    PausePanel, PendingKeyBind, SaveListAction,
    SaveListPanel, SaveListTitle, ScrollContainer, ScrollContent, SettingsAction,
    SettingsDropdownLabel, SettingsDropdownList, SettingsDropdownRoot, SettingsDropdownRow,
    SettingsGameplayGroup, SettingsKeyBindingsGroup, SettingsSlider, SettingsSliderFill,
    SettingsSliderKnob, SettingsTab, SettingsValue, SettingsValueText, StatusText,
    StatusTextKind, SlotArea, TeleportAction, BlockPanelText, BlockPanelTextKind,
    SettingsText, SettingsTextKind,
    UiPanelBinding, UiPanelId, UiRuntime,
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
