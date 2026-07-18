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

