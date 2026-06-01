use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::game::world::blocks::{BlockKind, SerializableBlockState, SerializedBlockState};
use crate::game::world::grid::WorldBlocks;

const TELEPORT_ENTRANCE_NAMES: &[&str] = &["Alpha In", "Beta In", "Gamma In", "Delta In"];
const TELEPORT_EXIT_NAMES: &[&str] = &["Alpha Out", "Beta Out", "Gamma Out", "Delta Out"];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeleportSettings {
    pub name: String,
    pub pair: Option<IVec3>,
}

impl TeleportSettings {
    pub fn unnamed(pos: IVec3) -> Self {
        Self {
            name: format!("Portal {}", pos_hash(pos)),
            pair: None,
        }
    }
}

impl Default for TeleportSettings {
    fn default() -> Self {
        Self::unnamed(IVec3::ZERO)
    }
}

impl SerializableBlockState for TeleportSettings {
    const BLOCK_KINDS: &'static [BlockKind] =
        &[BlockKind::TeleportEntrance, BlockKind::TeleportExit];
}

pub(crate) fn default_state(
    kind: BlockKind,
    _pos: IVec3,
    world: &WorldBlocks,
) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&TeleportSettings {
        name: next_teleport_name(kind, world),
        pair: None,
    })
}

pub(crate) fn normalize_state(
    state: &SerializedBlockState,
    _pos: IVec3,
) -> Option<SerializedBlockState> {
    SerializedBlockState::from_state(&state.decode::<TeleportSettings>()?)
}

pub(crate) fn settings(world: &WorldBlocks, pos: IVec3) -> TeleportSettings {
    world
        .block_state(pos)
        .unwrap_or_else(|| TeleportSettings::unnamed(pos))
}

pub(crate) fn set_settings(world: &mut WorldBlocks, pos: IVec3, settings: TeleportSettings) {
    world.set_block_state(pos, settings);
}

pub(crate) fn clear_pair_references(pos: IVec3, world: &mut WorldBlocks) {
    let positions: Vec<IVec3> = world.system_blocks.keys().copied().collect();
    for other in positions {
        let mut settings = settings(world, other);
        if settings.pair == Some(pos) {
            settings.pair = None;
            set_settings(world, other, settings);
        }
    }
}

fn next_teleport_name(kind: BlockKind, world: &WorldBlocks) -> String {
    let base_names = match kind {
        BlockKind::TeleportEntrance => TELEPORT_ENTRANCE_NAMES,
        BlockKind::TeleportExit => TELEPORT_EXIT_NAMES,
        _ => &[],
    };
    let used: HashSet<String> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, block)| (block.kind == kind).then(|| settings(world, *pos).name))
        .collect();

    for name in base_names {
        if !used.contains(*name) {
            return (*name).to_owned();
        }
    }

    for index in 2.. {
        for name in base_names {
            let candidate = format!("{name} {index}");
            if !used.contains(&candidate) {
                return candidate;
            }
        }
    }
    unreachable!()
}

fn pos_hash(pos: IVec3) -> i32 {
    pos.x.abs() * 31 + pos.y.abs() * 17 + pos.z.abs() * 13
}
