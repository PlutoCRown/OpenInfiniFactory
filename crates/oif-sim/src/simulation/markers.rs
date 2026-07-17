use glam::IVec3;

use crate::blocks::{BlockData, BlockKind, MarkerBehavior};
use crate::world::grid::WorldBlocks;

/// 清除并重建全部静态生成的虚拟 marker（焊点 / 钻头 / 机身占格等）
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

/// 按 MarkerBehavior 在目标格写入生成 marker
fn place_generated_marker(world: &mut WorldBlocks, origin: IVec3, marker: MarkerBehavior) {
    let (offset, kind, facing) = match marker {
        MarkerBehavior::WeldPoint { offset, facing } => {
            (offset, BlockKind::WeldPoint, facing)
        }
        MarkerBehavior::DrillHead { offset, facing } => {
            (offset, BlockKind::DrillHead, facing)
        }
        MarkerBehavior::RollerBody { facing } => (IVec3::ZERO, BlockKind::RollerBody, facing),
        MarkerBehavior::StamperBody { facing } => (IVec3::ZERO, BlockKind::StamperBody, facing),
    };

    let pos = origin + offset;
    // 无碰撞 marker 进 system_blocks；有碰撞机身进 blocks，仅要求平台层可放（可与宿主 System 同格）
    let can_place = match marker {
        MarkerBehavior::WeldPoint { .. } | MarkerBehavior::DrillHead { .. } => {
            world.can_place_virtual_block_at(pos)
        }
        MarkerBehavior::RollerBody { .. } | MarkerBehavior::StamperBody { .. } => {
            pos.y >= 0 && world.can_place_platform_at(pos)
        }
    };
    if can_place {
        world.insert(pos, BlockData::new(kind, facing));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::Facing;

    #[test]
    fn stamper_body_occupies_cell_against_materials() {
        let pos = IVec3::new(2, 1, 0);
        let mut world = WorldBlocks::default();
        world.insert(pos, BlockData::new(BlockKind::Stamper, Facing::East));
        refresh_static_generated_markers(&mut world);

        let body = world.blocks.get(&pos).expect("StamperBody in blocks layer");
        assert_eq!(body.kind, BlockKind::StamperBody);
        assert_eq!(body.facing, Facing::East);
        assert!(body.kind.has_collision());

        assert!(world.is_occupied(pos));
        assert!(!world.can_move_into(pos));
        assert!(!world.can_place_block_kind_at(pos, BlockKind::Material));
        assert!(!world.can_place_platform_at(pos));
    }

    #[test]
    fn roller_body_occupies_cell_against_materials() {
        let pos = IVec3::new(3, 1, 1);
        let mut world = WorldBlocks::default();
        world.insert(pos, BlockData::new(BlockKind::Roller, Facing::North));
        refresh_static_generated_markers(&mut world);

        let body = world.blocks.get(&pos).expect("RollerBody in blocks layer");
        assert_eq!(body.kind, BlockKind::RollerBody);
        assert!(world.is_occupied(pos));
        assert!(!world.can_move_into(pos));
        assert!(!world.can_place_block_kind_at(pos, BlockKind::Material));
    }
}
