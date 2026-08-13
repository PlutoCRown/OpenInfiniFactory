use glam::IVec3;

use crate::blocks::{BlockData, BlockKind, MarkerBehavior};
use crate::world::direction::Facing;
use crate::world::grid::WorldBlocks;

/// 清除并重建全部静态生成的虚拟 marker（焊点 / 钻头等）
pub fn refresh_static_generated_markers(world: &mut WorldBlocks) {
    if world.marker_source_count == 0 && world.generated_marker_count == 0 {
        return;
    }
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

#[cfg(test)]
mod tests {
    use super::refresh_static_generated_markers;
    use crate::blocks::{BlockData, BlockKind};
    use crate::world::direction::Facing;
    use crate::world::grid::WorldBlocks;
    use glam::IVec3;

    /// 验证最后一个 marker 来源删除后，遗留生成块仍会被清理
    #[test]
    fn removing_last_marker_source_clears_generated_marker() {
        let mut world = WorldBlocks::default();
        let origin = IVec3::ZERO;
        let marker_pos = IVec3::X;
        world.insert(origin, BlockData::new(BlockKind::Welder, Facing::East));

        refresh_static_generated_markers(&mut world);
        assert_eq!(world.marker_source_count, 1);
        assert_eq!(world.generated_marker_count, 1);
        assert_eq!(
            world.system_blocks.get(&marker_pos).map(|block| block.kind),
            Some(BlockKind::WeldPoint)
        );

        world.remove(&origin);
        refresh_static_generated_markers(&mut world);
        assert_eq!(world.marker_source_count, 0);
        assert_eq!(world.generated_marker_count, 0);
        assert!(!world.system_blocks.contains_key(&marker_pos));
    }
}
