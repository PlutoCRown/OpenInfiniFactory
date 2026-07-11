use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::blocks::{BlockData, BlockId, BlockKind};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{
    BlockSettings, MaterialFace, MaterialFaceMark, MaterialWeld, WorldBlocks,
};

/// 方块所在层：工厂/材料或系统层
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockLayer {
    Factory,
    System,
}

/// 单格编辑前后快照
#[derive(Clone, Debug, PartialEq)]
pub struct CellSnapshot {
    pub block: BlockData,
    pub layer: BlockLayer,
    pub settings: Option<BlockSettings>,
}

/// 单格前后差异
#[derive(Clone, Debug, PartialEq)]
pub struct CellDelta {
    pub pos: IVec3,
    pub before: Option<CellSnapshot>,
    pub after: Option<CellSnapshot>,
}

/// 面标记前后差异
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceMarkDelta {
    pub face: MaterialFace,
    pub before: Option<MaterialFaceMark>,
    pub after: Option<MaterialFaceMark>,
}

/// 系统方块设置前后差异
#[derive(Clone, Debug, PartialEq)]
pub struct SettingsDelta {
    pub pos: IVec3,
    pub before: Option<BlockSettings>,
    pub after: Option<BlockSettings>,
}

/// 可正向/反向应用的世界补丁
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldPatch {
    pub cells: Vec<CellDelta>,
    pub welds_add: Vec<MaterialWeld>,
    pub welds_remove: Vec<MaterialWeld>,
    pub face_marks: Vec<FaceMarkDelta>,
    pub settings: Vec<SettingsDelta>,
}

impl WorldPatch {
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
            && self.welds_add.is_empty()
            && self.welds_remove.is_empty()
            && self.face_marks.is_empty()
            && self.settings.is_empty()
    }

    pub fn affected_positions(&self) -> HashSet<IVec3> {
        let mut positions = HashSet::new();
        for delta in &self.cells {
            positions.insert(delta.pos);
            if let Some(after) = &delta.after {
                positions.insert(delta.pos);
                let _ = after;
            }
            if let Some(before) = &delta.before {
                positions.insert(delta.pos);
                let _ = before;
            }
        }
        for delta in &self.settings {
            positions.insert(delta.pos);
        }
        positions
    }

    pub fn touches_goal_or_generator(&self) -> bool {
        self.cells.iter().any(|delta| {
            delta.before.as_ref().is_some_and(|snap| {
                snap.block.kind == BlockKind::Goal || snap.block.kind == BlockKind::Generator
            }) || delta.after.as_ref().is_some_and(|snap| {
                snap.block.kind == BlockKind::Goal || snap.block.kind == BlockKind::Generator
            })
        }) || self.settings.iter().any(|delta| {
            matches!(
                (&delta.before, &delta.after),
                (Some(BlockSettings::Generator(_)), _)
                    | (_, Some(BlockSettings::Generator(_)))
                    | (Some(BlockSettings::Goal(_)), _)
                    | (_, Some(BlockSettings::Goal(_)))
            )
        })
    }

    pub fn apply_forward(&self, world: &mut WorldBlocks) {
        self.apply(world, true);
    }

    pub fn apply_inverse(&self, world: &mut WorldBlocks) {
        self.apply(world, false);
    }

    fn apply(&self, world: &mut WorldBlocks, forward: bool) {
        for delta in &self.settings {
            let value = if forward {
                delta.after.clone()
            } else {
                delta.before.clone()
            };
            match value {
                Some(settings) => {
                    world.block_settings.insert(delta.pos, settings);
                    world.topology_revision = world.topology_revision.wrapping_add(1);
                }
                None => {
                    world.block_settings.remove(&delta.pos);
                    world.topology_revision = world.topology_revision.wrapping_add(1);
                }
            }
        }

        for delta in &self.cells {
            let snapshot = if forward {
                delta.after.clone()
            } else {
                delta.before.clone()
            };
            apply_cell_snapshot(world, delta.pos, snapshot);
        }

        let (add, remove) = if forward {
            (&self.welds_add, &self.welds_remove)
        } else {
            (&self.welds_remove, &self.welds_add)
        };
        for weld in remove {
            world.material_welds.remove(weld);
        }
        for weld in add {
            if world.material_welds.insert(*weld) {
                world.topology_revision = world.topology_revision.wrapping_add(1);
            }
        }

        for delta in &self.face_marks {
            let value = if forward {
                delta.after
            } else {
                delta.before
            };
            match value {
                Some(mark) => {
                    world.material_face_marks.insert(delta.face, mark);
                }
                None => {
                    world.material_face_marks.remove(&delta.face);
                }
            }
        }

        if self.touches_goal_or_generator() {
            world.resync_acceptor_structures();
        }
    }
}

/// 读取单格当前快照
pub fn capture_cell(world: &WorldBlocks, pos: IVec3) -> Option<CellSnapshot> {
    if let Some(&block) = world.system_blocks.get(&pos) {
        return Some(CellSnapshot {
            block,
            layer: BlockLayer::System,
            settings: world.block_settings.get(&pos).cloned(),
        });
    }
    world.blocks.get(&pos).copied().map(|block| CellSnapshot {
        block,
        layer: BlockLayer::Factory,
        settings: None,
    })
}

/// 从补丁涉及的方块实例收集焊缝
pub fn capture_welds_for_ids(world: &WorldBlocks, ids: &HashSet<BlockId>) -> Vec<MaterialWeld> {
    if ids.is_empty() {
        return Vec::new();
    }
    world
        .material_welds
        .iter()
        .filter(|weld| ids.contains(&weld.a) || ids.contains(&weld.b))
        .copied()
        .collect()
}

/// 从补丁涉及的方块实例收集面标记
pub fn capture_face_marks_for_ids(
    world: &WorldBlocks,
    ids: &HashSet<BlockId>,
) -> Vec<FaceMarkDelta> {
    if ids.is_empty() {
        return Vec::new();
    }
    world
        .material_face_marks
        .iter()
        .filter(|(face, _)| ids.contains(&face.block))
        .map(|(face, mark)| FaceMarkDelta {
            face: *face,
            before: Some(*mark),
            after: Some(*mark),
        })
        .collect()
}

pub fn weld_diff(before: &[MaterialWeld], after: &[MaterialWeld]) -> (Vec<MaterialWeld>, Vec<MaterialWeld>) {
    let before_set: HashSet<_> = before.iter().copied().collect();
    let after_set: HashSet<_> = after.iter().copied().collect();
    let add = after
        .iter()
        .filter(|weld| !before_set.contains(weld))
        .copied()
        .collect();
    let remove = before
        .iter()
        .filter(|weld| !after_set.contains(weld))
        .copied()
        .collect();
    (add, remove)
}

pub fn face_mark_diff(
    before: &[FaceMarkDelta],
    after: &[FaceMarkDelta],
) -> Vec<FaceMarkDelta> {
    let before_map: HashMap<MaterialFace, Option<MaterialFaceMark>> = before
        .iter()
        .map(|delta| (delta.face, delta.before))
        .collect();
    let after_map: HashMap<MaterialFace, Option<MaterialFaceMark>> = after
        .iter()
        .map(|delta| (delta.face, delta.after))
        .collect();
    let mut faces: HashSet<MaterialFace> = before_map.keys().copied().collect();
    faces.extend(after_map.keys().copied());
    faces
        .into_iter()
        .filter_map(|face| {
            let b = before_map.get(&face).copied().flatten();
            let a = after_map.get(&face).copied().flatten();
            (b != a).then_some(FaceMarkDelta {
                face,
                before: b,
                after: a,
            })
        })
        .collect()
}

pub fn block_ids_from_snapshots(snapshots: impl Iterator<Item = Option<CellSnapshot>>) -> HashSet<BlockId> {
    snapshots
        .flatten()
        .map(|snap| snap.block.id)
        .filter(|id| !id.is_none())
        .collect()
}

pub fn build_cell_patch(
    world: &mut WorldBlocks,
    positions: &[IVec3],
    mutate: impl FnOnce(&mut WorldBlocks),
) -> WorldPatch {
    let before_cells: HashMap<IVec3, Option<CellSnapshot>> = positions
        .iter()
        .map(|pos| (*pos, capture_cell(world, *pos)))
        .collect();
    let before_ids = block_ids_from_snapshots(before_cells.values().cloned());
    let welds_before = capture_welds_for_ids(world, &before_ids);
    let face_marks_before = capture_face_marks_for_ids(world, &before_ids);

    mutate(world);

    let mut cells = Vec::with_capacity(positions.len());
    let mut after_ids = HashSet::new();
    for pos in positions {
        let after = capture_cell(world, *pos);
        if let Some(ref snap) = after {
            if !snap.block.id.is_none() {
                after_ids.insert(snap.block.id);
            }
        }
            cells.push(CellDelta {
            pos: *pos,
            before: before_cells[pos].clone(),
            after,
        });
    }
    after_ids.extend(before_ids);
    let welds_after = capture_welds_for_ids(world, &after_ids);
    let face_marks_after = capture_face_marks_for_ids(world, &after_ids);
    let (welds_add, welds_remove) = weld_diff(&welds_before, &welds_after);
    let face_marks = face_mark_diff(&face_marks_before, &face_marks_after);

    WorldPatch {
        cells,
        welds_add,
        welds_remove,
        face_marks,
        settings: Vec::new(),
    }
}

pub fn build_settings_patch(
    pos: IVec3,
    before: Option<BlockSettings>,
    after: Option<BlockSettings>,
) -> WorldPatch {
    WorldPatch {
        settings: vec![SettingsDelta { pos, before, after }],
        ..Default::default()
    }
}

pub fn build_relocate_patch(world: &WorldBlocks, moves: &[(IVec3, IVec3)]) -> WorldPatch {
    let mut cells = Vec::new();
    for (from, to) in moves {
        let Some(before) = capture_cell(world, *from) else {
            continue;
        };
        cells.push(CellDelta {
            pos: *from,
            before: Some(before.clone()),
            after: None,
        });
        cells.push(CellDelta {
            pos: *to,
            before: capture_cell(world, *to),
            after: Some(before),
        });
    }
    WorldPatch {
        cells,
        welds_add: Vec::new(),
        welds_remove: Vec::new(),
        face_marks: Vec::new(),
        settings: Vec::new(),
    }
}

pub fn build_rotation_patch(
    pos: IVec3,
    original: CellSnapshot,
    current_facing: Facing,
) -> Option<WorldPatch> {
    if original.block.facing == current_facing {
        return None;
    }
    let mut after_block = original.block;
    after_block.facing = current_facing;
    let after = CellSnapshot {
        block: after_block,
        layer: original.layer,
        settings: original.settings.clone(),
    };
    Some(WorldPatch {
        cells: vec![CellDelta {
            pos,
            before: Some(original),
            after: Some(after),
        }],
        ..Default::default()
    })
}

fn apply_cell_snapshot(world: &mut WorldBlocks, pos: IVec3, snapshot: Option<CellSnapshot>) {
    let _ = world.system_blocks.remove(&pos);
    let removed_factory = world.blocks.remove(&pos);
    if removed_factory.is_some() || world.block_settings.contains_key(&pos) {
        world.block_settings.remove(&pos);
    }

    let Some(snapshot) = snapshot else {
        world.topology_revision = world.topology_revision.wrapping_add(1);
        return;
    };

    let mut block = snapshot.block;
    world.assign_block_id(&mut block);
    match snapshot.layer {
        BlockLayer::Factory => {
            world.blocks.insert(pos, block);
        }
        BlockLayer::System => {
            world.system_blocks.insert(pos, block);
            if let Some(settings) = snapshot.settings {
                world.block_settings.insert(pos, settings);
            } else if let Some(default_settings) = block.kind.default_settings(pos) {
                world.block_settings.insert(pos, default_settings);
            }
        }
    }
    world.topology_revision = world.topology_revision.wrapping_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind};
    use crate::game::world::direction::Facing;

    #[test]
    fn place_and_undo_round_trip() {
        let mut world = WorldBlocks::default();
        let pos = IVec3::new(1, 0, 0);
        let block = BlockData::new(BlockKind::Conveyor, Facing::North);

        let patch = build_cell_patch(&mut world, &[pos], |world| {
            world.insert(pos, block);
        });
        assert!(capture_cell(&world, pos).is_some());

        patch.apply_inverse(&mut world);
        assert!(capture_cell(&world, pos).is_none());

        patch.apply_forward(&mut world);
        let restored = capture_cell(&world, pos).expect("block restored");
        assert_eq!(restored.block.kind, BlockKind::Conveyor);
    }

    #[test]
    fn rotation_patch_round_trip() {
        let mut world = WorldBlocks::default();
        let pos = IVec3::ZERO;
        world.insert(pos, BlockData::new(BlockKind::Conveyor, Facing::North));
        let original = capture_cell(&world, pos).expect("cell");

        let patch = build_rotation_patch(pos, original, Facing::East).expect("patch");
        patch.apply_forward(&mut world);
        assert_eq!(world.blocks[&pos].facing, Facing::East);

        patch.apply_inverse(&mut world);
        assert_eq!(world.blocks[&pos].facing, Facing::North);
    }
}
