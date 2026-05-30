use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::game::world::blocks::{BlockData, BlockKind, MaterialKind, StampColor};
use crate::game::world::direction::Facing;

pub const REACH: f32 = 12.0;
pub const FLOOR_RADIUS: i32 = 12;
const TELEPORT_ENTRANCE_NAMES: &[&str] = &["Alpha In", "Beta In", "Gamma In", "Delta In"];
const TELEPORT_EXIT_NAMES: &[&str] = &["Alpha Out", "Beta Out", "Gamma Out", "Delta Out"];

#[derive(Resource, Default, Clone)]
pub struct WorldBlocks {
    pub blocks: HashMap<IVec3, BlockData>,
    pub system_blocks: HashMap<IVec3, BlockData>,
    pub material_welds: HashSet<MaterialWeld>,
    pub material_face_marks: HashMap<MaterialFace, MaterialFaceMark>,
    pub block_settings: HashMap<IVec3, BlockSettings>,
    pub topology_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockSettings {
    Generator(GeneratorSettings),
    Labeler(LabelerSettings),
    Converter(ConverterSettings),
    Teleport(TeleportSettings),
}

impl BlockSettings {
    fn matches_kind(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Generator(_), Self::Generator(_))
                | (Self::Labeler(_), Self::Labeler(_))
                | (Self::Converter(_), Self::Converter(_))
                | (Self::Teleport(_), Self::Teleport(_))
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneratorSettings {
    pub period: u64,
    pub material: MaterialKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LabelerSettings {
    pub color: StampColor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConverterSettings {
    pub mode: ConverterMode,
    pub input: MaterialKind,
    pub output: MaterialKind,
}

impl Default for ConverterSettings {
    fn default() -> Self {
        Self {
            mode: ConverterMode::AnyInput,
            input: MaterialKind::Basic,
            output: MaterialKind::Iron,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConverterMode {
    AnyInput,
    SpecificInput,
}

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

impl ConverterMode {
    pub fn name_key(self) -> &'static str {
        match self {
            Self::AnyInput => "converter.mode.any",
            Self::SpecificInput => "converter.mode.specific",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::AnyInput => Self::SpecificInput,
            Self::SpecificInput => Self::AnyInput,
        }
    }
}

impl Default for LabelerSettings {
    fn default() -> Self {
        Self {
            color: StampColor::Red,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MaterialFace {
    pub pos: IVec3,
    pub normal: IVec3,
}

impl MaterialFace {
    pub fn new(pos: IVec3, normal: IVec3) -> Self {
        Self { pos, normal }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MaterialFaceMark {
    pub color: StampColor,
    pub source: MaterialFaceMarkSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MaterialFaceMarkSource {
    Stamper,
    Roller,
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
        let previous = if block.kind.is_system_layer() {
            self.system_blocks.insert(pos, block)
        } else {
            self.blocks.insert(pos, block)
        };
        if !self.block_settings.contains_key(&pos) {
            if let Some(mut settings) = block.kind.default_settings(pos) {
                if let BlockSettings::Teleport(teleport_settings) = &mut settings {
                    teleport_settings.name = self.next_teleport_name(block.kind);
                }
                self.block_settings.insert(pos, settings);
            }
        }
        if previous != Some(block) {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        previous
    }

    pub fn remove(&mut self, pos: &IVec3) -> Option<BlockData> {
        let removed = self.blocks.remove(pos);
        if removed.is_some() {
            self.material_welds.retain(|weld| !weld.contains(*pos));
            self.material_face_marks.retain(|face, _| face.pos != *pos);
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        removed
    }

    pub fn remove_system(&mut self, pos: &IVec3) -> Option<BlockData> {
        let removed = self.system_blocks.remove(pos);
        if removed.is_some() {
            self.block_settings.remove(pos);
            for settings in self.block_settings.values_mut() {
                if let BlockSettings::Teleport(settings) = settings {
                    if settings.pair == Some(*pos) {
                        settings.pair = None;
                    }
                }
            }
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        removed
    }

    pub fn clear(&mut self) {
        if !self.blocks.is_empty() || !self.system_blocks.is_empty() {
            self.blocks.clear();
            self.system_blocks.clear();
            self.material_welds.clear();
            self.material_face_marks.clear();
            self.block_settings.clear();
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
            self.material_face_marks
                .retain(|face, _| self.blocks.contains_key(&face.pos));
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn generator_settings(&self, pos: IVec3) -> GeneratorSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Generator(settings)) => *settings,
            _ => GeneratorSettings::default(),
        }
    }

    pub fn set_block_settings(&mut self, pos: IVec3, settings: BlockSettings) {
        let Some(block) = self.system_blocks.get(&pos).copied() else {
            return;
        };
        let Some(default_settings) = block.kind.default_settings(pos) else {
            return;
        };
        if !settings.matches_kind(&default_settings) {
            return;
        }
        if self.block_settings.get(&pos) != Some(&settings) {
            self.block_settings.insert(pos, settings);
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn set_generator_settings(&mut self, pos: IVec3, settings: GeneratorSettings) {
        self.set_block_settings(pos, BlockSettings::Generator(settings));
    }

    pub fn labeler_settings(&self, pos: IVec3) -> LabelerSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Labeler(settings)) => *settings,
            _ => LabelerSettings::default(),
        }
    }

    pub fn set_labeler_settings(&mut self, pos: IVec3, settings: LabelerSettings) {
        self.set_block_settings(pos, BlockSettings::Labeler(settings));
    }

    pub fn converter_settings(&self, pos: IVec3) -> ConverterSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Converter(settings)) => *settings,
            _ => ConverterSettings::default(),
        }
    }

    pub fn set_converter_settings(&mut self, pos: IVec3, settings: ConverterSettings) {
        self.set_block_settings(pos, BlockSettings::Converter(settings));
    }

    pub fn teleport_settings(&self, pos: IVec3) -> TeleportSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Teleport(settings)) => settings.clone(),
            _ => TeleportSettings::unnamed(pos),
        }
    }

    pub fn set_teleport_settings(&mut self, pos: IVec3, settings: TeleportSettings) {
        self.set_block_settings(pos, BlockSettings::Teleport(settings));
    }

    pub fn cycle_teleport_pair(&mut self, pos: IVec3) {
        let Some(block) = self.system_blocks.get(&pos).copied() else {
            return;
        };
        let target_kind = match block.kind {
            BlockKind::TeleportEntrance => BlockKind::TeleportExit,
            BlockKind::TeleportExit => BlockKind::TeleportEntrance,
            _ => return,
        };
        let mut candidates: Vec<IVec3> = self
            .system_blocks
            .iter()
            .filter_map(|(candidate_pos, candidate)| {
                (candidate.kind == target_kind).then_some(*candidate_pos)
            })
            .collect();
        candidates.sort_by_key(|candidate| self.teleport_settings(*candidate).name);

        let current = self.teleport_settings(pos).pair;
        let next = if candidates.is_empty() {
            None
        } else {
            let index = current
                .and_then(|current| {
                    candidates
                        .iter()
                        .position(|candidate| *candidate == current)
                })
                .map(|index| (index + 1) % candidates.len())
                .unwrap_or(0);
            Some(candidates[index])
        };

        let mut settings = self.teleport_settings(pos);
        settings.pair = next;
        self.set_teleport_settings(pos, settings);
    }

    fn next_teleport_name(&self, kind: BlockKind) -> String {
        let base_names = match kind {
            BlockKind::TeleportEntrance => TELEPORT_ENTRANCE_NAMES,
            BlockKind::TeleportExit => TELEPORT_EXIT_NAMES,
            _ => &[],
        };
        let used: HashSet<String> = self
            .block_settings
            .iter()
            .filter_map(|(pos, settings)| {
                if !self
                    .system_blocks
                    .get(pos)
                    .is_some_and(|block| block.kind == kind)
                {
                    return None;
                }
                match settings {
                    BlockSettings::Teleport(settings) => Some(settings.name.clone()),
                    _ => None,
                }
            })
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

    pub fn set_material_face_mark(&mut self, face: MaterialFace, mark: MaterialFaceMark) {
        if !self.is_material_at(face.pos) {
            return;
        }
        if self.material_face_marks.insert(face, mark) != Some(mark) {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn replace_material_face_marks(&mut self, marks: HashMap<MaterialFace, MaterialFaceMark>) {
        if self.material_face_marks != marks {
            self.material_face_marks = marks;
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

    pub fn can_place_platform_at(&self, pos: IVec3) -> bool {
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

    pub fn accepts_material_at(&self, pos: IVec3) -> bool {
        self.system_blocks
            .get(&pos)
            .is_some_and(|block| block.kind.accepts_material())
    }

    pub fn clear_generated_markers(&mut self) {
        let blocks_before = self.blocks.len();
        self.blocks
            .retain(|_, block| !block.kind.is_generated_marker());
        let system_before = self.system_blocks.len();
        self.system_blocks
            .retain(|_, block| !block.kind.is_generated_marker());
        if self.blocks.len() != blocks_before || self.system_blocks.len() != system_before {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
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
                    kind: BlockKind::Stone,
                    facing: Facing::North,
                },
            );
        }
    }
}

pub fn grid_to_world(pos: IVec3) -> Vec3 {
    pos.as_vec3() + Vec3::splat(0.5)
}

fn pos_hash(pos: IVec3) -> i32 {
    pos.x.abs() * 31 + pos.y.abs() * 17 + pos.z.abs() * 13
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
    for pos in world.system_blocks.keys() {
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
