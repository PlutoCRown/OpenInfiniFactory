use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::world::blocks::{BlockData, BlockKind, Facing};

pub const REACH: f32 = 8.0;
pub const FLOOR_RADIUS: i32 = 12;

#[derive(Resource, Default)]
pub struct WorldBlocks {
    pub blocks: HashMap<IVec3, BlockData>,
    pub topology_revision: u64,
}

impl WorldBlocks {
    pub fn insert(&mut self, pos: IVec3, block: BlockData) -> Option<BlockData> {
        let previous = self.blocks.insert(pos, block);
        if previous != Some(block) {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        previous
    }

    pub fn remove(&mut self, pos: &IVec3) -> Option<BlockData> {
        let removed = self.blocks.remove(pos);
        if removed.is_some() {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        removed
    }

    pub fn clear(&mut self) {
        if !self.blocks.is_empty() {
            self.blocks.clear();
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&IVec3, &BlockData) -> bool) {
        let before = self.blocks.len();
        self.blocks.retain(|pos, block| keep(pos, block));
        if self.blocks.len() != before {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn is_occupied(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.has_collision())
    }

    pub fn can_place_solid_at(&self, pos: IVec3) -> bool {
        !self.is_occupied(pos)
    }

    pub fn clear_generated_markers(&mut self) {
        self.retain(|_, block| !block.kind.is_generated_marker());
    }
}

#[derive(Clone, Copy)]
pub struct TargetHit {
    pub pos: IVec3,
    pub normal: IVec3,
}

pub fn seed_demo_world(world: &mut WorldBlocks) {
    for x in -FLOOR_RADIUS..=FLOOR_RADIUS {
        for z in -FLOOR_RADIUS..=FLOOR_RADIUS {
            world.insert(
                IVec3::new(x, 0, z),
                BlockData {
                    kind: BlockKind::Solid,
                    facing: Facing::North,
                },
            );
        }
    }

    world.insert(
        IVec3::new(0, 1, 0),
        BlockData {
            kind: BlockKind::Conveyor,
            facing: Facing::East,
        },
    );
    world.insert(
        IVec3::new(1, 1, 0),
        BlockData {
            kind: BlockKind::Piston,
            facing: Facing::South,
        },
    );
    world.insert(
        IVec3::new(2, 1, 0),
        BlockData {
            kind: BlockKind::Goal,
            facing: Facing::North,
        },
    );
}

pub fn grid_to_world(pos: IVec3) -> Vec3 {
    pos.as_vec3() + Vec3::splat(0.5)
}

pub fn raycast_blocks(origin: Vec3, dir: Vec3, world: &WorldBlocks) -> Option<TargetHit> {
    let mut best: Option<(f32, TargetHit)> = None;

    for pos in world.blocks.keys() {
        let center = grid_to_world(*pos);
        let min = center - Vec3::splat(0.5);
        let max = center + Vec3::splat(0.5);
        if let Some((distance, normal)) = ray_aabb(origin, dir, min, max) {
            if distance <= REACH && best.map_or(true, |(best_distance, _)| distance < best_distance)
            {
                best = Some((distance, TargetHit { pos: *pos, normal }));
            }
        }
    }

    best.map(|(_, hit)| hit)
}

fn ray_aabb(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<(f32, IVec3)> {
    let mut t_min = 0.0;
    let mut t_max = REACH;
    let mut normal = IVec3::ZERO;

    for axis in 0..3 {
        let origin_axis = origin[axis];
        let dir_axis = dir[axis];
        let min_axis = min[axis];
        let max_axis = max[axis];

        if dir_axis.abs() < 0.0001 {
            if origin_axis < min_axis || origin_axis > max_axis {
                return None;
            }
            continue;
        }

        let inv_dir = 1.0 / dir_axis;
        let mut t1 = (min_axis - origin_axis) * inv_dir;
        let mut t2 = (max_axis - origin_axis) * inv_dir;
        let mut axis_normal = IVec3::ZERO;

        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
            axis_normal[axis] = 1;
        } else {
            axis_normal[axis] = -1;
        }

        if t1 > t_min {
            t_min = t1;
            normal = axis_normal;
        }
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    Some((t_min.max(0.0), normal))
}
