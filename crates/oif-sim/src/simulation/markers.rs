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
    match marker {
        // 无碰撞 marker 进 system_blocks
        MarkerBehavior::WeldPoint { .. } | MarkerBehavior::DrillHead { .. } => {
            if world.can_place_virtual_block_at(pos) {
                world.insert(pos, BlockData::new(kind, facing));
            }
        }
        // 有碰撞机身进 machine_bodies：可与 System 宿主、印花材料同格
        MarkerBehavior::RollerBody { .. } | MarkerBehavior::StamperBody { .. } => {
            if pos.y >= 0 && !world.blocks_factory_or_scene_at(pos) {
                world
                    .machine_bodies
                    .insert(pos, BlockData::new(kind, facing));
                world.topology_revision = world.topology_revision.wrapping_add(1);
            }
        }
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

        let body = world
            .machine_bodies
            .get(&pos)
            .expect("StamperBody in machine_bodies");
        assert_eq!(body.kind, BlockKind::StamperBody);
        assert_eq!(body.facing, Facing::East);
        assert!(body.kind.has_collision());
        assert!(!world.blocks.contains_key(&pos));

        assert!(world.is_occupied(pos));
        assert!(!world.can_move_into(pos));
        assert!(!world.can_place_block_kind_at(pos, BlockKind::material("basic")));
        assert!(!world.can_place_platform_at(pos));
    }

    #[test]
    fn roller_body_occupies_cell_against_materials() {
        let pos = IVec3::new(3, 1, 1);
        let mut world = WorldBlocks::default();
        world.insert(pos, BlockData::new(BlockKind::Roller, Facing::North));
        refresh_static_generated_markers(&mut world);

        let body = world
            .machine_bodies
            .get(&pos)
            .expect("RollerBody in machine_bodies");
        assert_eq!(body.kind, BlockKind::RollerBody);
        assert!(world.is_occupied(pos));
        assert!(!world.can_move_into(pos));
        assert!(!world.can_place_block_kind_at(pos, BlockKind::material("basic")));
    }

    #[test]
    fn stamper_body_coexists_with_stamp_material_in_blocks() {
        let pos = IVec3::new(1, 1, 0);
        let mut world = WorldBlocks::default();
        world.insert(pos, BlockData::new(BlockKind::Stamper, Facing::West));
        world.insert(pos, BlockData::new(BlockKind::stamp("red"), Facing::East));
        refresh_static_generated_markers(&mut world);

        assert!(world.machine_bodies.contains_key(&pos));
        assert_eq!(
            world.blocks.get(&pos).map(|b| b.kind),
            Some(BlockKind::stamp("red"))
        );
        assert!(world.is_occupied(pos));
    }
}
