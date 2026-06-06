use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Drag, DragEnd, DragStart, Out, Over, Pointer};
use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    StartMenuScreen, UiPanelId,
};
use crate::game::world::grid::WorldBlocks;
use crate::shared::config::{ActionKeyName, GameConfig};
use crate::shared::i18n::I18n;
use crate::shared::save::SaveState;

use super::types::{
    ConfirmDialogState, Crosshair, GameplayHudVisibility, InGameHudStyle, InGameHudVisibility,
    InlineTextEditState, InventoryItems, LocalizedText, MenuAction, OpenBlockPanelDropdown,
    OpenSettingsDropdown, PanelCloseButton, PanelDragState, PanelPosition, PanelTitleBar,
    PanelVisibility, PanelWindow, PendingKeyBind, SettingsTab, StatusText, StatusTextKind,
    TextPromptRoot, UiHoverState, UiPanelBinding, UiRuntime,
};
include!("font.rs");
include!("status.rs");
include!("hover.rs");
include!("localized.rs");
include!("panels.rs");
include!("hud.rs");
