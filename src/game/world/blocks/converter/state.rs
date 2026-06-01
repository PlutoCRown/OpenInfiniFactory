use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::world::blocks::{
    BlockKind, MaterialKind, SerializableBlockState, SerializedBlockState,
};
use crate::game::world::grid::WorldBlocks;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConverterSettings {
    pub mode: ConverterMode,
    pub input: MaterialKind,
    pub output: MaterialKind,
}

impl Default for ConverterSettings {
    fn default() -> Self {
        Self {
            mode: ConverterMode::AnyInput,
            input: MaterialKind::Basic,
            output: MaterialKind::Iron,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConverterMode {
    AnyInput,
    SpecificInput,
}

impl SerializableBlockState for ConverterSettings {
    const BLOCK_KINDS: &'static [BlockKind] = &[BlockKind::Converter];
}

pub(super) fn default_state(_pos: IVec3, _world: &WorldBlocks) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&ConverterSettings::default())
}

pub(super) fn normalize_state(
    state: &SerializedBlockState,
    _pos: IVec3,
) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&state.decode::<ConverterSettings>()?)
}

pub(crate) fn settings(world: &WorldBlocks, pos: IVec3) -> ConverterSettings {
    world.block_state(pos).unwrap_or_default()
}

pub(crate) fn set_settings(world: &mut WorldBlocks, pos: IVec3, settings: ConverterSettings) {
    world.set_block_state(pos, settings);
}
