use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::world::blocks::{
    BlockKind, MaterialKind, SerializableBlockState, SerializedBlockState, DEFAULT_GENERATOR_PERIOD,
};
use crate::game::world::grid::WorldBlocks;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneratorSettings {
    pub period: u64,
    pub material: MaterialKind,
}

impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            period: DEFAULT_GENERATOR_PERIOD,
            material: MaterialKind::Basic,
        }
    }
}

impl SerializableBlockState for GeneratorSettings {
    const BLOCK_KINDS: &'static [BlockKind] = &[BlockKind::Generator];
}

pub(super) fn default_state(_pos: IVec3, _world: &WorldBlocks) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&GeneratorSettings::default())
}

pub(super) fn normalize_state(
    state: &SerializedBlockState,
    _pos: IVec3,
) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&state.decode::<GeneratorSettings>()?)
}

pub(crate) fn settings(world: &WorldBlocks, pos: IVec3) -> GeneratorSettings {
    world.block_state(pos).unwrap_or_default()
}

pub(crate) fn set_settings(world: &mut WorldBlocks, pos: IVec3, settings: GeneratorSettings) {
    world.set_block_state(pos, settings);
}
