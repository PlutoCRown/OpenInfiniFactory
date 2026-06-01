use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Drag, DragEnd, DragStart, Out, Over, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, SimulationState, SolutionState, TeleportRenameState,
    WorldEntryMode,
};
use crate::game::ui::OpenBlockPanelDropdown;
use crate::game::world::grid::WorldBlocks;
use crate::shared::config::{ConfigAction, GameConfig};
use crate::shared::i18n::I18n;
use crate::shared::save::{SaveKind, SaveState};

use super::types::{
    Crosshair, GameplayHudVisibility, InGameHudStyle, InGameHudVisibility, InventoryItems,
    LocalizedText, OpenSettingsDropdown, PanelCloseButton, PanelDragState, PanelPosition,
    PanelText, PanelTextKind, PanelTitleBar, PanelVisibility, PanelWindow, PauseMenuAction,
    PendingKeyBind, SettingsTab, StatusText, StatusTextKind, TextPromptRoot, UiHoverState,
    UiPanelBinding, UiPanelContext, UiPanelId, UiRuntime,
};
include!("font.rs");
include!("status.rs");
include!("hover.rs");
include!("localized.rs");
include!("panels.rs");
include!("hud.rs");
