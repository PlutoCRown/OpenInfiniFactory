pub(super) fn gravity_moves(
    world: &WorldBlocks,
    structures: &mut StructureState,
    skip_factory_positions: &HashSet<IVec3>,
    hard_pusher_head_occupancy: &HashSet<IVec3>,
    suction: &SuctionLinks,
) -> Vec<StructureMove> {
    let ids = structures.gravity_structure_ids();
    let mut moves = Vec::new();
    let mut handled = HashSet::new();

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
        let Some(structure) =
            structures.linked_expand_pusher_subset(suction, positions, IVec3::NEG_Y)
        else {
            handled.extend(positions.iter().copied());
            continue;
        };
        handled.extend(structure.iter().copied());

        if structure
            .iter()
            .any(|pos| skip_factory_positions.contains(pos))
        {
            continue;
        }
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
        // 整块结构一起下落；不可拆开，否则焊接材料会被撕开并丢掉焊缝
        if can_move_gravity_structure(
            world,
            &structure,
            structures,
            hard_pusher_head_occupancy,
            suction,
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
        } else {
            structures.record_gravity_support(id, world, hard_pusher_head_occupancy);
        }
    }
    moves
}
