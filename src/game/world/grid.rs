use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::game::world::blocks::{BlockData, BlockKind, MaterialKind};
use crate::game::world::direction::Facing;

pub const REACH: f32 = 12.0;
pub const FLOOR_RADIUS: i32 = 12;

#[derive(Resource, Default, Clone)]
pub struct WorldBlocks {
    pub blocks: HashMap<IVec3, BlockData>,
    pub material_welds: HashSet<MaterialWeld>,
    pub generator_settings: HashMap<IVec3, GeneratorSettings>,
    pub topology_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneratorSettings {
    pub period: u64,
    pub material: MaterialKind,
}

impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            period: crate::game::world::blocks::DEFAULT_GENERATOR_PERIOD,
            material: MaterialKind::Basic,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MaterialWeld {
    pub a: IVec3,
    pub b: IVec3,
}

impl MaterialWeld {
    pub fn new(a: IVec3, b: IVec3) -> Self {
        if (a.x, a.y, a.z) <= (b.x, b.y, b.z) {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }

    pub fn other(self, pos: IVec3) -> Option<IVec3> {
        if self.a == pos {
            Some(self.b)
        } else if self.b == pos {
            Some(self.a)
        } else {
            None
        }
    }

    pub fn contains(self, pos: IVec3) -> bool {
        self.a == pos || self.b == pos
    }
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
            self.material_welds.retain(|weld| !weld.contains(*pos));
            self.generator_settings.remove(pos);
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        removed
    }

    pub fn clear(&mut self) {
        if !self.blocks.is_empty() {
            self.blocks.clear();
            self.material_welds.clear();
            self.generator_settings.clear();
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&IVec3, &BlockData) -> bool) {
        let before = self.blocks.len();
        self.blocks.retain(|pos, block| keep(pos, block));
        if self.blocks.len() != before {
            self.material_welds.retain(|weld| {
                self.blocks.contains_key(&weld.a) && self.blocks.contains_key(&weld.b)
            });
            self.generator_settings
                .retain(|pos, _| self.blocks.contains_key(pos));
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn generator_settings(&self, pos: IVec3) -> GeneratorSettings {
        self.generator_settings
            .get(&pos)
            .copied()
            .unwrap_or_default()
    }

    pub fn set_generator_settings(&mut self, pos: IVec3, settings: GeneratorSettings) {
        if !self
            .blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_generator())
        {
            return;
        }
        if self.generator_settings.insert(pos, settings) != Some(settings) {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn weld_materials(&mut self, a: IVec3, b: IVec3) {
        if a == b || !self.is_material_at(a) || !self.is_material_at(b) {
            return;
        }
        if self.material_welds.insert(MaterialWeld::new(a, b)) {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn replace_material_welds(&mut self, welds: HashSet<MaterialWeld>) {
        if self.material_welds != welds {
            self.material_welds = welds;
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

    pub fn is_material_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_material())
    }

    pub fn is_factory_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_factory())
    }

    pub fn is_scene_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_scene())
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
    let dir = dir.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    let mut best: Option<(f32, TargetHit)> = None;

    for (pos, block) in &world.blocks {
        if !block.kind.has_collision() {
            continue;
        }

        let center = grid_to_world(*pos);
        let min = center - Vec3::splat(0.5);
        let max = center + Vec3::splat(0.5);
        if let Some((distance, normal)) = ray_aabb(origin, dir, min, max, REACH) {
            if best.map_or(true, |(best_distance, _)| distance < best_distance) {
                best = Some((distance, TargetHit { pos: *pos, normal }));
            }
        }
    }

    best.map(|(_, hit)| hit)
}

fn ray_aabb(
    origin: Vec3,
    dir: Vec3,
    min: Vec3,
    max: Vec3,
    max_distance: f32,
) -> Option<(f32, IVec3)> {
    let mut t_enter = 0.0;
    let mut t_exit = max_distance;
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
        let mut near = (min_axis - origin_axis) * inv_dir;
        let mut far = (max_axis - origin_axis) * inv_dir;
        let near_normal = if inv_dir >= 0.0 {
            -axis_vec(axis)
        } else {
            axis_vec(axis)
        };

        if near > far {
            std::mem::swap(&mut near, &mut far);
        }

        if near > t_enter {
            t_enter = near;
            normal = near_normal;
        }
        t_exit = t_exit.min(far);
        if t_enter > t_exit {
            return None;
        }
    }

    if t_exit < 0.0 {
        None
    } else {
        Some((t_enter.max(0.0), normal))
    }
}

fn axis_vec(axis: usize) -> IVec3 {
    match axis {
        0 => IVec3::X,
        1 => IVec3::Y,
        _ => IVec3::Z,
    }
}
