use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Drag, DragEnd, DragStart, Out, Over, Pointer};
use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, StartMenuScreen,
    UiPanelId,
};
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::SaveState;

use crate::game::ui::access::{UiMainThread, i18n};
use crate::game::ui::components::ui_logical_bounds;

use super::types::{
    CarriedItem, Crosshair, GameplayHudVisibility, InGameHudStyle, InGameHudVisibility,
    InlineTextEditState, InventoryItems, LocalizedText, OpenBlockPanelDropdown,
    OpenSettingsDropdown, PanelCloseButton, PanelDragState, PanelFlowLayout, PanelPosition,
    PanelTitleBar, PanelVisibility, PanelWindow, PendingKeyBind, SettingsTab, StatusText,
    StatusTextKind, TextPromptRoot, TextPromptState, UiHost, UiHoverState, UiPanelBinding,
    UiRuntime,
};
use crate::game::ui::core::confirm_dialog::{ConfirmButtonId, ConfirmDialogState};
include!("font.rs");
include!("status.rs");
include!("hover.rs");
include!("localized.rs");
include!("panels.rs");
include!("hud.rs");
