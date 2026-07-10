use bevy::prelude::*;

use crate::game::blocks::{BlockData, BlockKind, MarkerBehavior};
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
        place_generated_marker(world, pos, marker);
    }
}

pub(super) fn run_static_marker_phase(world: &mut WorldBlocks) {
    refresh_static_generated_markers(world);
}

fn place_generated_marker(world: &mut WorldBlocks, origin: IVec3, marker: MarkerBehavior) {
    let (offset, kind, facing, platform_collision) = match marker {
        MarkerBehavior::WeldPoint { offset, facing } => {
            (offset, BlockKind::WeldPoint, facing, false)
        }
        MarkerBehavior::DrillHead { offset, facing } => {
            (offset, BlockKind::DrillHead, facing, false)
        }
    };

    let pos = origin + offset;
    let can_place = if platform_collision {
        world.can_place_blocks_layer_at(pos, BlockKind::Platform)
    } else {
        world.can_place_virtual_block_at(pos)
    };
    if can_place {
        world.insert(pos, BlockData::new(kind, facing));
    }
}
