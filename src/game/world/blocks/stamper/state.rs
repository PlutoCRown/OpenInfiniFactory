use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::world::blocks::{
    BlockKind, SerializableBlockState, SerializedBlockState, StampColor,
};
use crate::game::world::grid::WorldBlocks;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StamperSettings {
    pub color: StampColor,
}

impl Default for StamperSettings {
    fn default() -> Self {
        Self {
            color: StampColor::Red,
        }
    }
}

impl SerializableBlockState for StamperSettings {
    const BLOCK_KINDS: &'static [BlockKind] = &[BlockKind::Stamper];
}

pub(super) fn default_state(_pos: IVec3, _world: &WorldBlocks) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&StamperSettings::default())
}

pub(super) fn normalize_state(
    state: &SerializedBlockState,
    _pos: IVec3,
) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&state.decode::<StamperSettings>()?)
}
