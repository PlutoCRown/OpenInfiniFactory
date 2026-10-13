use super::{
    HashMap, HashSet, IVec3, MovementMark, StructureId, StructureMove, StructureState,
    SuctionLinks, WorldBlocks, can_move_pusher_heads, expanded_move_structure,
    hard_pusher_head_blocked_below, own_extended_heads, structure_id_gravity_grounded,
    structure_supported_by_lifter,
};

/// 标记未被稳定支撑或抬升器托住的结构下落
pub(in crate::simulation) fn gravity_moves(
    world: &WorldBlocks,
    structures: &mut StructureState,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
    suction: &SuctionLinks,
) -> Vec<StructureMove> {
    let ids = structures.gravity_structure_ids();
    let mut moves = Vec::new();
    let mut handled = HashSet::new();
    let mut grounded_memo: HashMap<StructureId, bool> = HashMap::new();

    for id in ids {
        let Some(positions) = structures.structure_positions(id) else {
            continue;
        };
        let Some(&sample) = positions.iter().next() else {
            continue;
        };
        if handled.contains(&sample) {
            continue;
        }

        // 经吸盘扩展；粘到固定结构则整体不可落（种子为完整结构时并集等价）
        let Some(structure) = structures.expand_unanchored_subset(suction, positions) else {
            handled.extend(positions.iter().copied());
            continue;
        };
        handled.extend(structure.iter().copied());

        if structure_supported_by_lifter(world, &structure) {
            for pos in &structure {
                if let Some(sid) = structures.id_at(*pos) {
                    structures.clear_gravity_support(sid);
                }
            }
            continue;
        }
        if structures.gravity_support_valid(id, world, hard_pusher_head_occupancy) {
            continue;
        }
        // 场景或锚定结构提供稳定支撑，找到一处即可早退并缓存
        if structures.try_record_stable_gravity_support(id, world, hard_pusher_head_occupancy) {
            continue;
        }
        // 未锚定结构的支撑链若最终接地，则本回合不落，无需展开整个移动集合
        if structure_id_gravity_grounded(
            world,
            structures,
            id,
            hard_pusher_head_occupancy,
            &mut grounded_memo,
        ) {
            continue;
        }
        // 整块结构一起下落；不可拆开，否则焊接材料会被撕开并丢掉焊缝
        let own = own_extended_heads(world, &structure, hard_pusher_head_occupancy);
        if hard_pusher_head_blocked_below(world, &structure, hard_pusher_head_occupancy, &own) {
            continue;
        }
        let Some(expanded) = expanded_move_structure(
            world,
            &structure,
            IVec3::NEG_Y,
            structures,
            super::collision::MovementExpansionMode::Gravity,
            suction,
        ) else {
            continue;
        };
        let own = own_extended_heads(world, &expanded, hard_pusher_head_occupancy);
        if can_move_pusher_heads(
            world,
            &expanded,
            IVec3::NEG_Y,
            hard_pusher_head_occupancy,
            &own,
        ) {
            for pos in &structure {
                if let Some(sid) = structures.id_at(*pos) {
                    structures.clear_gravity_support(sid);
                }
            }
            moves.push(StructureMove::translate_marked(
                id,
                structure,
                IVec3::NEG_Y,
                MovementMark::Vertical,
            ));
        }
    }
    moves
}
