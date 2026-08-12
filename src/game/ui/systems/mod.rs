use bevy::ecs::system::SystemParam;
use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Drag, DragEnd, DragStart, Out, Over, Pointer};
use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, SimulationState, SolutionState, UiPanelId,
};
use crate::game::systems::gameplay::AimFocus;
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::SaveState;

use crate::game::ui::access::{UiMainThread, i18n};
use crate::game::ui::components::ui_logical_bounds;

use super::types::{
    CarriedItem, Crosshair, DropdownSurface, GameplayHudVisibility, InGameHudStyle,
    InGameHudVisibility, InlineTextEditState, InventoryItem, InventoryItems, LocalizedText,
    OpenBlockPanelDropdown, OpenSettingsDropdown, PanelCloseButton, PanelDragState,
    PanelFlowLayout, PanelPosition, PanelTitleBar, PanelVisibility, PanelWindow, PendingKeyBind,
    SettingsTab, StatusText, StatusTextKind, TextPromptRoot, TextPromptState, UiHost, UiHoverState,
    UiNavigation, UiPanelBinding,
};
use crate::game::ui::core::StartMenuPage;
use crate::game::ui::core::confirm_dialog::{ConfirmButtonId, ConfirmDialogState};
include!("font.rs");
include!("icons.rs");
include!("status.rs");
include!("hover.rs");
include!("localized.rs");
include!("panels.rs");
include!("hud.rs");
