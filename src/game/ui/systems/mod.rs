use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Drag, DragEnd, DragStart, Out, Over, Pointer};
use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, StartMenuScreen,
    UiPanelId,
};
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::SaveState;

use crate::game::ui::access::{i18n, I18nRevision, UiMainThread};
use crate::game::ui::components::ui_logical_bounds;

use super::types::{
    CarriedItem, Crosshair, GameplayHudVisibility, InGameHudStyle, InGameHudVisibility,
    InlineTextEditState, InventoryItems, LocalizedText, OpenBlockPanelDropdown,
    OpenSettingsDropdown, PanelCloseButton, PanelDragState, PanelPosition, PanelTitleBar,
    PanelVisibility, PanelWindow, PendingKeyBind, SettingsTab, StatusText, StatusTextKind,
    TextPromptRoot, UiHost, UiHoverState, UiPanelBinding, UiRuntime,
};
include!("font.rs");
include!("status.rs");
include!("hover.rs");
include!("localized.rs");
include!("panels.rs");
include!("hud.rs");
