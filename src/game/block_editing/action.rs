use bevy::prelude::*;

use crate::game::world::blocks::{MaterialKind, StampColor};

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum BlockPanelAction {
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
    ToggleTeleportPairDropdown,
    SetTeleportPair(Option<IVec3>),
    StartTeleportRename,
}

impl BlockPanelAction {
    pub fn label_key(self) -> &'static str {
        match self {
            Self::PeriodDown => "button.period_down",
            Self::PeriodUp => "button.period_up",
            Self::ToggleMaterialDropdown | Self::SetMaterial(_) => "button.material_next",
            Self::ToggleColorDropdown | Self::SetColor(_) => "button.next_color",
            Self::ToggleInputDropdown | Self::SetInput(_) => "button.input_material",
            Self::ToggleOutputDropdown | Self::SetOutput(_) => "button.output_material",
            Self::ToggleTeleportPairDropdown | Self::SetTeleportPair(_) => "button.teleport_pair",
            Self::StartTeleportRename => "button.teleport_rename",
        }
    }

    pub fn mutates_world(self) -> bool {
        !matches!(
            self,
            Self::ToggleMaterialDropdown
                | Self::ToggleColorDropdown
                | Self::ToggleInputDropdown
                | Self::ToggleOutputDropdown
                | Self::ToggleTeleportPairDropdown
        )
    }
}
