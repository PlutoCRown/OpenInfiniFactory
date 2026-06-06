use bevy::prelude::*;

use super::action::BlockPanelAction;
use crate::game::world::blocks::MaterialKind;
use crate::game::world::grid::WorldBlocks;
use crate::shared::i18n::I18n;

#[derive(Resource, Default)]
pub struct OpenBlockPanelDropdown(pub Option<BlockPanelDropdown>);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockPanelDropdown {
    GeneratorMaterial,
    GoalMaterial,
    LabelerColor,
    ConverterInput,
    ConverterOutput,
    TeleportPair,
}

impl BlockPanelDropdown {
    pub fn toggle_action(self) -> BlockPanelAction {
        use BlockPanelAction::*;
        match self {
            Self::GeneratorMaterial | Self::GoalMaterial => ToggleMaterialDropdown,
            Self::LabelerColor => ToggleColorDropdown,
            Self::ConverterInput => ToggleInputDropdown,
            Self::ConverterOutput => ToggleOutputDropdown,
            Self::TeleportPair => ToggleTeleportPairDropdown,
        }
    }

    pub fn uses_material_icon(self) -> bool {
        matches!(
            self,
            Self::GeneratorMaterial
                | Self::GoalMaterial
                | Self::ConverterInput
                | Self::ConverterOutput
        )
    }

    pub fn is_dynamic(self) -> bool {
        matches!(self, Self::TeleportPair)
    }

    pub fn selected_label(
        self,
        active_pos: Option<IVec3>,
        world: &WorldBlocks,
        i18n: &I18n,
    ) -> String {
        let Some(pos) = active_pos else {
            return String::new();
        };
        match self {
            Self::GeneratorMaterial => i18n.text(world.generator_settings(pos).material.name_key()),
            Self::GoalMaterial => i18n.text(world.goal_settings(pos).material.name_key()),
            Self::LabelerColor => i18n.text(world.labeler_settings(pos).color.name_key()),
            Self::ConverterInput => i18n.text(world.converter_settings(pos).input.name_key()),
            Self::ConverterOutput => i18n.text(world.converter_settings(pos).output.name_key()),
            Self::TeleportPair => active_pos
                .and_then(|pos| world.teleport_settings(pos).pair)
                .map(|pair| world.teleport_settings(pair).name)
                .unwrap_or_else(|| i18n.text("teleport.none")),
        }
    }

    pub fn selected_material(self, active_pos: Option<IVec3>, world: &WorldBlocks) -> Option<MaterialKind> {
        let pos = active_pos?;
        match self {
            Self::GeneratorMaterial => Some(world.generator_settings(pos).material),
            Self::GoalMaterial => Some(world.goal_settings(pos).material),
            Self::ConverterInput => Some(world.converter_settings(pos).input),
            Self::ConverterOutput => Some(world.converter_settings(pos).output),
            Self::LabelerColor | Self::TeleportPair => None,
        }
    }
}
