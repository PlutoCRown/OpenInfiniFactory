use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::world::blocks::{BlockData, BlockKind, MarkerBehavior};
use crate::game::world::grid::WorldBlocks;

pub fn refresh_static_generated_markers(world: &mut WorldBlocks) {
    world.clear_generated_markers();

    let markers: Vec<(IVec3, MarkerBehavior)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .marker_behavior(block.facing)
                .map(|marker| (*pos, marker))
        })
        .collect();

    for (pos, marker) in markers {
        if matches!(marker, MarkerBehavior::BlockerHead { .. }) {
            continue;
        }
        place_generated_marker(world, pos, marker);
    }
}

pub(super) fn run_static_marker_phase(world: &mut WorldBlocks) {
    refresh_static_generated_markers(world);
}

pub(super) fn run_powered_marker_phase(world: &mut WorldBlocks, powered_devices: &HashSet<IVec3>) {
    let markers: Vec<(IVec3, MarkerBehavior)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            block
                .kind
                .marker_behavior(block.facing)
                .map(|marker| (*pos, marker))
        })
        .collect();

    for (pos, marker) in markers {
        if !matches!(marker, MarkerBehavior::BlockerHead { .. }) {
            continue;
        }
        if !powered_devices.contains(&pos) {
            place_generated_marker(world, pos, marker);
        }
    }
}

fn place_generated_marker(world: &mut WorldBlocks, origin: IVec3, marker: MarkerBehavior) {
    let (offset, kind, facing, platform_collision) = match marker {
        MarkerBehavior::WeldPoint { offset, facing } => {
            (offset, BlockKind::WeldPoint, facing, false)
        }
        MarkerBehavior::BlockerHead { offset, facing } => {
            (offset, BlockKind::BlockerHead, facing, true)
        }
        MarkerBehavior::DrillHead { offset, facing } => {
            (offset, BlockKind::DrillHead, facing, false)
        }
    };

    let pos = origin + offset;
    let can_place = if platform_collision {
        world.can_place_platform_at(pos)
    } else {
        !world.system_blocks.contains_key(&pos)
    };
    if can_place {
        world.insert(pos, BlockData { kind, facing });
    }
}
