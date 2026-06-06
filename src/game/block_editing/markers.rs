use bevy::prelude::*;

use crate::game::world::blocks::MaterialKind;

use super::dropdown::BlockPanelDropdown;

#[derive(Component, Clone, Copy)]
pub struct BlockPanelTitle;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockPanelText(pub BlockPanelTextKind);

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockPanelTextKind {
    GeneratorPeriod,
    TeleportName,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockPanelDropdownLabel(pub BlockPanelDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockPanelDropdownList(pub BlockPanelDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockMaterialIconSlot(pub BlockPanelDropdown);

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub struct BlockMaterialIcon(pub MaterialKind);
