use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::world::blocks::{
    BlockKind, MaterialKind, SerializableBlockState, SerializedBlockState,
};
use crate::game::world::grid::WorldBlocks;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GoalSettings {
    pub material: MaterialKind,
}

impl Default for GoalSettings {
    fn default() -> Self {
        Self {
            material: MaterialKind::Basic,
        }
    }
}

impl SerializableBlockState for GoalSettings {
    const BLOCK_KINDS: &'static [BlockKind] = &[BlockKind::Goal];
}

pub(super) fn default_state(_pos: IVec3, _world: &WorldBlocks) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&GoalSettings::default())
}

pub(super) fn normalize_state(
    state: &SerializedBlockState,
    _pos: IVec3,
) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&state.decode::<GoalSettings>()?)
}
