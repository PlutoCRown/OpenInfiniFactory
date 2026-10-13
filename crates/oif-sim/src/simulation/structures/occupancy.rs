use super::{
    BlockKind, HashMap, HashSet, IVec3, MovementMark, StructureMove, StructureState, SuctionLinks,
    WorldBlocks, expanded_move_structure_with_occupancy, movement_expansion_mode,
};

/// 回合内格子速度声明：同格同速度兼容，异速冲突
#[derive(Default, Clone, Debug)]
pub(in crate::simulation) struct MovingOccupancy {
    /// 格子对应已接受运动的位移
    claims: HashMap<IVec3, IVec3>,
}

impl MovingOccupancy {
    /// 查询格子已经登记的运动位移
    pub(super) fn velocity_at(&self, cell: IVec3) -> Option<IVec3> {
        self.claims.get(&cell).copied()
    }
}

/// 按原顺序反复仲裁普通运动直到无进展，推杆保留给执行阶段裁决
pub(in crate::simulation) fn arbitrate_movement_plan(
    world: &WorldBlocks,
    structures: &StructureState,
    suction: &SuctionLinks,
    candidates: Vec<StructureMove>,
) -> Vec<StructureMove> {
    let mut occupancy = MovingOccupancy::default();
    let mut pending: Vec<usize> = candidates
        .iter()
        .enumerate()
        .filter_map(|(idx, movement)| {
            (!matches!(
                movement,
                StructureMove::Translate {
                    mark: MovementMark::Push,
                    ..
                }
            ))
            .then_some(idx)
        })
        .collect();

    loop {
        let mut progress = false;
        let mut still = Vec::new();
        for idx in pending {
            let footprint = match &candidates[idx] {
                StructureMove::Translate {
                    structure,
                    offset,
                    mark,
                    source,
                    ..
                } => {
                    let Some(expanded) = expanded_move_structure_with_occupancy(
                        world,
                        structure,
                        *offset,
                        structures,
                        movement_expansion_mode(*mark, *source),
                        suction,
                        Some(&occupancy),
                    ) else {
                        still.push(idx);
                        continue;
                    };
                    // 脆弱成员也必须满足目标占位，不能只依赖推动链的让出判定。
                    if *offset != IVec3::ZERO
                        && !expanded.iter().all(|pos| {
                            let target = *pos + *offset;
                            if target.y < 0 {
                                return false;
                            }
                            if expanded.contains(&target) {
                                return true;
                            }
                            if let Some(existing) = occupancy.velocity_at(target) {
                                return existing == *offset;
                            }
                            world.can_move_into_yielding_fragile(target)
                                || world
                                    .blocks
                                    .get(&target)
                                    .is_some_and(|block| block.kind == BlockKind::PusherHead)
                        })
                    {
                        still.push(idx);
                        continue;
                    }
                    // 平移同时登记起点和终点，重复格子的位移相同，无需排序去重。
                    let mut footprint = Vec::with_capacity(expanded.len() * 2);
                    if *offset != IVec3::ZERO {
                        for pos in expanded {
                            footprint.push((pos, *offset));
                            footprint.push((pos + *offset, *offset));
                        }
                    }
                    footprint
                }
                StructureMove::Rotate { structure, .. } => {
                    structure.iter().map(|&pos| (pos, IVec3::ZERO)).collect()
                }
            };
            // 先验证整份占位再提交，失败候选不能留下部分声明。
            if footprint.iter().all(|(cell, velocity)| {
                occupancy
                    .claims
                    .get(cell)
                    .is_none_or(|existing| existing == velocity)
            }) {
                occupancy.claims.extend(footprint);
                progress = true;
            } else {
                still.push(idx);
            }
        }
        pending = still;
        if !progress || pending.is_empty() {
            break;
        }
    }

    let reject: HashSet<usize> = pending.into_iter().collect();
    candidates
        .into_iter()
        .enumerate()
        .filter_map(|(idx, movement)| (!reject.contains(&idx)).then_some(movement))
        .collect()
}
