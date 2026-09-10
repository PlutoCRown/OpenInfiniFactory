/// 合并重力与设备运动标签：抬升覆盖重力；其余重叠保留，按优先级排序，执行时再 fallback
pub(super) fn merge_structure_movement_plan(
    mut planned_moves: Vec<StructureMove>,
    device_moves: Vec<StructureMove>,
    history: &mut MovementHistory,
    structures: &StructureState,
    world: &WorldBlocks,
) -> Vec<StructureMove> {
    let living_structures: HashSet<StructureId> = structures.structure_ids().collect();
    let living_blocks: HashSet<BlockId> = world.blocks.values().map(|block| block.id).collect();
    history.prune_missing(&living_structures, &living_blocks);
    planned_moves.extend(device_moves);
    // 抬升标签本身表达「压住重力」：重叠格子上的重力标签直接丢掉
    let lift_positions: HashSet<IVec3> = planned_moves
        .iter()
        .filter_map(|movement| match movement {
            StructureMove::Translate {
                mark: MovementMark::Vertical,
                source: Some(_),
                structure,
                ..
            } => Some(structure.iter().copied()),
            _ => None,
        })
        .flatten()
        .collect();
    if !lift_positions.is_empty() {
        planned_moves.retain(|movement| {
            !matches!(
                movement,
                StructureMove::Translate {
                    mark: MovementMark::Vertical,
                    source: None,
                    structure,
                    ..
                } if structure.iter().any(|pos| lift_positions.contains(pos))
            )
        });
    }
    planned_moves.sort_by(|a, b| compare_movement_priority(a, b, history));
    // 同结构同位移的 Push 合并推杆，粘头也能同回合同步推
    coalesce_same_push_moves(planned_moves)
}

/// 把同 structure_id + offset 的 Push 合成一条，actors 并在一起
fn coalesce_same_push_moves(moves: Vec<StructureMove>) -> Vec<StructureMove> {
    let mut out = Vec::new();
    let mut push_index: HashMap<(StructureId, IVec3), usize> = HashMap::new();
    for movement in moves {
        match movement {
            StructureMove::Translate {
                structure_id,
                structure,
                offset,
                actors,
                mark: MovementMark::Push,
                source,
                source_pos,
            } => {
                let key = (structure_id, offset);
                if let Some(&i) = push_index.get(&key) {
                    if let StructureMove::Translate {
                        structure: existing,
                        actors: existing_actors,
                        ..
                    } = &mut out[i]
                    {
                        existing.extend(structure);
                        existing_actors.extend(actors);
                    }
                } else {
                    push_index.insert(key, out.len());
                    out.push(StructureMove::Translate {
                        structure_id,
                        structure,
                        offset,
                        actors,
                        mark: MovementMark::Push,
                        source,
                        source_pos,
                    });
                }
            }
            other => out.push(other),
        }
    }
    out
}

fn compare_movement_priority(
    a: &StructureMove,
    b: &StructureMove,
    history: &MovementHistory,
) -> Ordering {
    movement_priority_key(a, history).cmp(&movement_priority_key(b, history))
}

fn movement_priority_key(
    movement: &StructureMove,
    history: &MovementHistory,
) -> (u8, u32, ConveyorSourcePriority) {
    // 种类优先：活塞 > 抬升 > 下落 > 旋转 > 传送带
    (
        movement_kind_priority(movement),
        movement.source().map_or(0, |_| history.count(movement)),
        conveyor_source_priority(movement),
    )
}

fn movement_kind_priority(movement: &StructureMove) -> u8 {
    match movement {
        StructureMove::Translate {
            mark: MovementMark::Push,
            ..
        } => 0,
        // 抬升器：有 source 的竖直移动
        StructureMove::Translate {
            mark: MovementMark::Vertical,
            source: Some(_),
            ..
        } => 1,
        StructureMove::Translate {
            mark: MovementMark::Vertical,
            source: None,
            ..
        } => 2,
        StructureMove::Rotate { .. } => 3,
        StructureMove::Translate {
            mark: MovementMark::Conveyor,
            ..
        } => 4,
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct ConveyorSourcePriority {
    positive_x: i32,
    negative_x: i32,
    positive_y: i32,
    negative_y: i32,
    positive_z: i32,
    negative_z: i32,
}

fn conveyor_source_priority(movement: &StructureMove) -> ConveyorSourcePriority {
    let Some(source) = movement.source_pos() else {
        return ConveyorSourcePriority::neutral();
    };
    if !matches!(
        movement,
        StructureMove::Translate {
            mark: MovementMark::Conveyor,
            ..
        }
    ) {
        return ConveyorSourcePriority::neutral();
    }
    ConveyorSourcePriority {
        positive_x: -source.x,
        negative_x: source.x,
        positive_y: -source.y,
        negative_y: source.y,
        positive_z: -source.z,
        negative_z: source.z,
    }
}

impl ConveyorSourcePriority {
    fn neutral() -> Self {
        Self {
            positive_x: 0,
            negative_x: 0,
            positive_y: 0,
            negative_y: 0,
            positive_z: 0,
            negative_z: 0,
        }
    }
}
