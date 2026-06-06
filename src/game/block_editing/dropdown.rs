use bevy::prelude::*;

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
