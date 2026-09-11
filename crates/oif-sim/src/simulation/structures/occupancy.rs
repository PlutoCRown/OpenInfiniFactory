use super::{
    BlockKind, HashMap, HashSet, IVec3, MovementMark, PusherAnimationKind, StructureMove,
    StructureState, SuctionLinks, WorldBlocks, expanded_move_structure_with_occupancy,
    movement_expansion_mode,
};

// 移动占位：动画占双格（起点+终点），同向 claim 兼容

/// 回合内格子速度声明：同格同速度兼容，异速冲突
#[derive(Default, Clone, Debug)]
pub(in crate::simulation) struct MovingOccupancy {
    /// cell → velocity（已接受运动的占位）
    claims: HashMap<IVec3, IVec3>,
}

impl MovingOccupancy {
    pub(super) fn velocity_at(&self, cell: IVec3) -> Option<IVec3> {
        self.claims.get(&cell).copied()
    }

    /// 平移 footprint：每个成员占住起点与终点，速度均为 offset
    pub(super) fn translate_footprint(
        structure: &HashSet<IVec3>,
        offset: IVec3,
    ) -> Vec<(IVec3, IVec3)> {
        let mut out = Vec::with_capacity(structure.len() * 2);
        for &p in structure {
            out.push((p, offset));
            if offset != IVec3::ZERO {
                out.push((p + offset, offset));
            }
        }
        out
    }

    pub(super) fn compatible_with(&self, footprint: &[(IVec3, IVec3)]) -> bool {
        footprint.iter().all(|(cell, velocity)| {
            self.claims
                .get(cell)
                .is_none_or(|existing| existing == velocity)
        })
    }

    pub(super) fn claim(&mut self, footprint: &[(IVec3, IVec3)]) -> bool {
        if !self.compatible_with(footprint) {
            return false;
        }
        for &(cell, velocity) in footprint {
            self.claims.insert(cell, velocity);
        }
        true
    }

    /// 目标格是否允许以该速度进入：空 / 结构内 / 同向离开的 claim
    pub(super) fn cell_allows_entry(
        &self,
        world: &WorldBlocks,
        cell: IVec3,
        velocity: IVec3,
        moving: &HashSet<IVec3>,
    ) -> bool {
        if cell.y < 0 {
            return false;
        }
        if moving.contains(&cell) {
            return true;
        }
        if let Some(existing) = self.velocity_at(cell) {
            return existing == velocity;
        }
        world.cell_accepts_move_from(cell - velocity, cell)
            || !world.is_occupied(cell)
            || world.is_fragile_material_at(cell)
            || world
                .blocks
                .get(&cell)
                .is_some_and(|b| b.kind == BlockKind::PusherHead)
    }
}

/// 按列表顺序 + 同向 fixpoint 仲裁；返回可提交的候选子集（保序）
/// Push（推杆变形）仍交 execute 顺序兜底；占位仲裁覆盖重力/传送带等平移
pub(in crate::simulation) fn arbitrate_movement_plan(
    world: &WorldBlocks,
    structures: &StructureState,
    suction: &SuctionLinks,
    candidates: Vec<StructureMove>,
) -> Vec<StructureMove> {
    let mut occupancy = MovingOccupancy::default();
    let mut slots: Vec<Option<StructureMove>> = candidates.into_iter().map(Some).collect();
    let mut pending: Vec<usize> = Vec::new();
    let mut passthrough_push: Vec<usize> = Vec::new();

    for (idx, slot) in slots.iter().enumerate() {
        let Some(m) = slot else {
            continue;
        };
        match m {
            StructureMove::Translate {
                mark: MovementMark::Push,
                ..
            } => passthrough_push.push(idx),
            _ => pending.push(idx),
        }
    }

    loop {
        let mut progress = false;
        let mut still = Vec::new();
        for idx in pending {
            let Some(movement) = slots[idx].take() else {
                continue;
            };
            if try_reserve_movement(world, structures, suction, &mut occupancy, &movement) {
                slots[idx] = Some(movement);
                progress = true;
            } else {
                slots[idx] = Some(movement);
                still.push(idx);
            }
        }
        pending = still;
        if !progress || pending.is_empty() {
            break;
        }
    }

    let reject: HashSet<usize> = pending.into_iter().collect();
    // 推杆仍尝试登记头占位（伸/收同向），失败也保留交 execute
    for idx in passthrough_push {
        if let Some(movement) = slots[idx].as_ref() {
            let _ = try_reserve_movement(world, structures, suction, &mut occupancy, movement);
        }
    }

    slots
        .into_iter()
        .enumerate()
        .filter_map(|(i, m)| if reject.contains(&i) { None } else { m })
        .collect()
}

fn try_reserve_movement(
    world: &WorldBlocks,
    structures: &StructureState,
    suction: &SuctionLinks,
    occupancy: &mut MovingOccupancy,
    movement: &StructureMove,
) -> bool {
    match movement {
        StructureMove::Translate {
            structure,
            offset,
            actors,
            mark,
            source,
            ..
        } => {
            let mode = movement_expansion_mode(*mark, *source);
            let Some(expanded) = expanded_move_structure_with_occupancy(
                world,
                structure,
                *offset,
                structures,
                mode,
                suction,
                Some(occupancy),
            ) else {
                return false;
            };
            if *offset != IVec3::ZERO
                && !expanded.iter().all(|pos| {
                    occupancy.cell_allows_entry(world, *pos + *offset, *offset, &expanded)
                })
            {
                return false;
            }

            let mut footprint = if *offset == IVec3::ZERO {
                Vec::new()
            } else {
                MovingOccupancy::translate_footprint(&expanded, *offset)
            };
            for actor in actors {
                let Some(block) = world.blocks.get(&actor.pos) else {
                    continue;
                };
                let forward = block.facing.forward_ivec3();
                let head = actor.pos + forward;
                match actor.animation {
                    // 只占头格：伸出进入 / 收回离开，同向可共用；不占体格以免与邻杆异速冲突
                    PusherAnimationKind::Extend => {
                        footprint.push((head, forward));
                    }
                    PusherAnimationKind::Retract => {
                        footprint.push((head, -forward));
                    }
                }
            }
            footprint.sort_by_key(|(c, v)| (c.x, c.y, c.z, v.x, v.y, v.z));
            footprint.dedup();
            occupancy.claim(&footprint)
        }
        StructureMove::Rotate { structure, .. } => {
            let footprint: Vec<_> = structure.iter().map(|&p| (p, IVec3::ZERO)).collect();
            occupancy.claim(&footprint)
        }
    }
}
