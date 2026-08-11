use glam::IVec3;

use crate::blocks::{BlockData, BlockKind, MarkerBehavior};
use crate::world::direction::Facing;
use crate::world::grid::WorldBlocks;

/// 清除并重建全部静态生成的虚拟 marker（焊点 / 钻头等）
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
        .chain(world.system_blocks.iter().filter_map(|(pos, block)| {
            block
                .kind
                .marker_behavior(block.facing)
                .map(|marker| (*pos, marker))
        }))
        .collect();

    for (pos, marker) in markers {
        place_generated_marker(world, pos, marker);
    }
}

pub(super) fn run_static_marker_phase(world: &mut WorldBlocks) {
    refresh_static_generated_markers(world);
}

/// Marker 放置策略：偏移虚拟格
enum MarkerPlacement {
    OffsetVirtual {
        offset: IVec3,
        kind: BlockKind,
        facing: Facing,
    },
}

fn marker_placement(marker: MarkerBehavior) -> MarkerPlacement {
    match marker {
        MarkerBehavior::WeldPoint { offset, facing } => MarkerPlacement::OffsetVirtual {
            offset,
            kind: BlockKind::WeldPoint,
            facing,
        },
        MarkerBehavior::DrillHead { offset, facing } => MarkerPlacement::OffsetVirtual {
            offset,
            kind: BlockKind::DrillHead,
            facing,
        },
    }
}

/// 按 MarkerBehavior 在目标格写入生成 marker
fn place_generated_marker(world: &mut WorldBlocks, origin: IVec3, marker: MarkerBehavior) {
    match marker_placement(marker) {
        // 无碰撞 marker 进 system_blocks
        MarkerPlacement::OffsetVirtual {
            offset,
            kind,
            facing,
        } => {
            let pos = origin + offset;
            if world.can_place_virtual_block_at(pos) {
                world.insert(pos, BlockData::new(kind, facing));
            }
        }
    }
}
