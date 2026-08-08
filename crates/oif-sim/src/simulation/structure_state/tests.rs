use super::*;
    use crate::blocks::BlockData;
    use crate::world::Facing;

    fn place(world: &mut WorldBlocks, pos: IVec3, kind: BlockKind, facing: Facing) -> BlockId {
        world.insert(pos, BlockData::new(kind, facing));
        world.blocks.get(&pos).unwrap().id
    }

    fn preferred_nodes(structure: &Structure, body: BlockId, forward: bool) -> &[BlockId] {
        let idx = structure.action_to_groups.get(&(body, forward)).unwrap()[0];
        structure.deform_groups[idx as usize].nodes.as_slice()
    }

    fn candidate_node_sets(
        structure: &Structure,
        body: BlockId,
        forward: bool,
    ) -> Vec<Vec<BlockId>> {
        structure
            .action_to_groups
            .get(&(body, forward))
            .unwrap()
            .iter()
            .map(|idx| structure.deform_groups[*idx as usize].nodes.clone())
            .collect()
    }

    /// 图 A：三平行推杆 → 两组共轴 deform（体列成环丢掉独立 fork）
    #[test]
    fn deform_figure_a_three_parallel() {
        let mut world = WorldBlocks::default();
        let b1 = place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Pusher,
            Facing::East,
        );
        let _f1 = place(
            &mut world,
            IVec3::new(1, 0, 0),
            BlockKind::Platform,
            Facing::North,
        );
        let b2 = place(
            &mut world,
            IVec3::new(0, 0, 1),
            BlockKind::Pusher,
            Facing::East,
        );
        let _f2 = place(
            &mut world,
            IVec3::new(1, 0, 1),
            BlockKind::Platform,
            Facing::North,
        );
        let b3 = place(
            &mut world,
            IVec3::new(0, 0, 2),
            BlockKind::Pusher,
            Facing::East,
        );
        let _f3 = place(
            &mut world,
            IVec3::new(1, 0, 2),
            BlockKind::Platform,
            Facing::North,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sid = state.structure_id_at(IVec3::new(0, 0, 0)).unwrap();
        let structure = state.get(sid).unwrap();
        assert_eq!(structure.deform_groups.len(), 2);

        let fwd = structure.action_to_groups.get(&(b1, true)).unwrap().clone();
        let rev = structure
            .action_to_groups
            .get(&(b1, false))
            .unwrap()
            .clone();
        assert_eq!(fwd.len(), 1);
        assert_eq!(rev.len(), 1);
        assert_eq!(structure.action_to_groups.get(&(b2, true)), Some(&fwd));
        assert_eq!(structure.action_to_groups.get(&(b3, true)), Some(&fwd));
        assert_eq!(structure.action_to_groups.get(&(b2, false)), Some(&rev));
        assert_eq!(structure.action_to_groups.get(&(b3, false)), Some(&rev));

        let fwd_nodes = &structure.deform_groups[fwd[0] as usize].nodes;
        let rev_nodes = &structure.deform_groups[rev[0] as usize].nodes;
        assert_eq!(fwd_nodes.len(), 6); // 3 heads + 3 fronts
        assert_eq!(rev_nodes.len(), 3); // 3 bodies
        assert!(rev_nodes.contains(&b1) && rev_nodes.contains(&b2) && rev_nodes.contains(&b3));
    }

    /// 图 B：折角三杆；fork 可产生多候选，须含文档中的最大成功集
    #[test]
    fn deform_figure_b_bent() {
        let mut world = WorldBlocks::default();
        let b1 = place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Pusher,
            Facing::East,
        );
        let b2 = place(
            &mut world,
            IVec3::new(1, 0, 0),
            BlockKind::Pusher,
            Facing::South,
        );
        let b3 = place(
            &mut world,
            IVec3::new(1, 0, 1),
            BlockKind::Pusher,
            Facing::West,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sid = state.structure_id_at(IVec3::new(0, 0, 0)).unwrap();
        let structure = state.get(sid).unwrap();

        let h1 = *structure.head_of.get(&b1).unwrap();
        let h2 = *structure.head_of.get(&b2).unwrap();
        let h3 = *structure.head_of.get(&b3).unwrap();

        let fwd1_sets = candidate_node_sets(structure, b1, true);
        assert!(
            fwd1_sets.iter().any(|n| {
                n.contains(&h1)
                    && n.contains(&b2)
                    && n.contains(&h2)
                    && n.contains(&b3)
                    && n.contains(&h3)
                    && !n.contains(&b1)
            }),
            "missing full (1,true) set: {fwd1_sets:?}"
        );
        assert_eq!(preferred_nodes(structure, b1, false), &[b1]);

        let fwd3 = candidate_node_sets(structure, b2, true);
        assert!(fwd3.iter().any(|n| {
            n.contains(&h2) && n.contains(&b3) && n.contains(&h3) && !n.contains(&b2)
        }));
        let rev3 = candidate_node_sets(structure, b2, false);
        assert!(
            rev3.iter()
                .any(|n| n.contains(&b2) && n.contains(&b1) && n.contains(&h1))
        );

        let fwd5 = preferred_nodes(structure, b3, true);
        assert!(fwd5.contains(&h3) && !fwd5.contains(&b3));
        let rev5 = candidate_node_sets(structure, b3, false);
        assert!(rev5.iter().any(|n| {
            n.contains(&b3)
                && n.contains(&h2)
                && n.contains(&b2)
                && n.contains(&h1)
                && n.contains(&b1)
        }));
    }

    /// 贴脸对向：独立正推含对面体+背面；共轴小集可只有两头
    #[test]
    fn deform_face_to_face_includes_back_cargo() {
        let mut world = WorldBlocks::default();
        let south = place(
            &mut world,
            IVec3::new(0, 0, 0),
            BlockKind::Blocker,
            Facing::South,
        );
        let north = place(
            &mut world,
            IVec3::new(0, 0, 1),
            BlockKind::Blocker,
            Facing::North,
        );
        let cargo = place(
            &mut world,
            IVec3::new(0, 0, 2),
            BlockKind::Wire,
            Facing::North,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sid = state.structure_id_at(IVec3::new(0, 0, 1)).unwrap();
        let structure = state.get(sid).unwrap();
        let h_south = *structure.head_of.get(&south).unwrap();
        let h_north = *structure.head_of.get(&north).unwrap();

        let south_fwd = candidate_node_sets(structure, south, true);
        assert!(
            south_fwd.iter().any(|n| {
                n.contains(&h_south)
                    && n.contains(&h_north)
                    && n.contains(&north)
                    && n.contains(&cargo)
                    && !n.contains(&south)
            }),
            "missing independent south+: {south_fwd:?}"
        );
        assert!(
            south_fwd.iter().any(|n| {
                n.contains(&h_south)
                    && n.contains(&h_north)
                    && !n.contains(&north)
                    && !n.contains(&cargo)
            }),
            "missing coaxial heads-only south+: {south_fwd:?}"
        );

        let north_fwd = candidate_node_sets(structure, north, true);
        assert!(north_fwd.iter().any(|n| {
            n.contains(&h_south)
                && n.contains(&h_north)
                && n.contains(&south)
                && !n.contains(&north)
        }));

        assert_eq!(preferred_nodes(structure, south, false), &[south]);
        let rev_north = preferred_nodes(structure, north, false);
        assert!(rev_north.contains(&north) && rev_north.contains(&cargo));
    }

    /// 平行双杆正面列相连、体不相邻：共轴小集 + 各自独立拖对面体
    #[test]

    fn deform_parallel_gap_fork_independent() {
        let mut world = WorldBlocks::default();
        let a = place(
            &mut world,
            IVec3::new(-1, 2, -6),
            BlockKind::Pusher,
            Facing::East,
        );
        let b = place(
            &mut world,
            IVec3::new(-1, 2, -4),
            BlockKind::Blocker,
            Facing::East,
        );
        let p0 = place(
            &mut world,
            IVec3::new(0, 2, -6),
            BlockKind::Platform,
            Facing::North,
        );
        let p1 = place(
            &mut world,
            IVec3::new(0, 2, -5),
            BlockKind::Platform,
            Facing::North,
        );
        let p2 = place(
            &mut world,
            IVec3::new(0, 2, -4),
            BlockKind::Platform,
            Facing::North,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let structure = state
            .get(state.structure_id_at(IVec3::new(-1, 2, -4)).unwrap())
            .unwrap();
        let ha = *structure.head_of.get(&a).unwrap();
        let hb = *structure.head_of.get(&b).unwrap();

        let a_fwd = candidate_node_sets(structure, a, true);
        let b_fwd = candidate_node_sets(structure, b, true);
        assert!(
            a_fwd.len() >= 2 && b_fwd.len() >= 2,
            "a={a_fwd:?} b={b_fwd:?}"
        );

        let coaxial = [p0, p1, p2, ha, hb];
        assert!(
            a_fwd.iter().any(|n| {
                n.len() == 5
                    && coaxial.iter().all(|id| n.contains(id))
                    && !n.contains(&a)
                    && !n.contains(&b)
            }),
            "missing coaxial: {a_fwd:?}"
        );
        assert!(
            a_fwd.iter().any(|n| {
                n.len() == 6
                    && coaxial.iter().all(|id| n.contains(id))
                    && n.contains(&b)
                    && !n.contains(&a)
            }),
            "missing a independent: {a_fwd:?}"
        );
        assert!(
            b_fwd.iter().any(|n| {
                n.len() == 6
                    && coaxial.iter().all(|id| n.contains(id))
                    && n.contains(&a)
                    && !n.contains(&b)
            }),
            "missing b independent: {b_fwd:?}"
        );

        // 首选应是共轴小集，且两杆共享同一组
        let pref_a = structure.action_to_groups.get(&(a, true)).unwrap()[0];
        let pref_b = structure.action_to_groups.get(&(b, true)).unwrap()[0];
        assert_eq!(pref_a, pref_b);
        assert_eq!(structure.deform_groups[pref_a as usize].nodes.len(), 5);
    }

    /// 单活塞 2×2 方环：正反单切/全切均成环 → 不写任何 DeformGroup
    #[test]
    fn deform_single_pusher_square_loop_discards() {
        let mut world = WorldBlocks::default();
        place(
            &mut world,
            IVec3::new(13, 3, 12),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(14, 3, 12),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(14, 4, 12),
            BlockKind::Platform,
            Facing::North,
        );
        let body = place(
            &mut world,
            IVec3::new(13, 4, 12),
            BlockKind::Blocker,
            Facing::East,
        );

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let structure = state
            .get(state.structure_id_at(IVec3::new(13, 4, 12)).unwrap())
            .unwrap();
        assert!(
            structure.deform_groups.is_empty(),
            "cyclic single pusher must discard all deform actions; got {:?}",
            structure.deform_groups
        );
        assert!(structure.action_to_groups.is_empty());
        assert!(structure.action_to_groups.get(&(body, true)).is_none());
        assert!(structure.action_to_groups.get(&(body, false)).is_none());
        assert!(state.deform_sides(&world, IVec3::new(13, 4, 12)).is_none());
    }

    /// 对向环：单切成环 → 全切；西正推与东反推共轴
    #[test]
    fn deform_opposing_ring_coaxial_after_all_cut() {
        let mut world = WorldBlocks::default();
        let west = IVec3::new(13, 2, 16);
        let east = IVec3::new(13, 4, 16);
        for (x, y, z) in [
            (12, 2, 16),
            (12, 3, 16),
            (12, 4, 16),
            (14, 2, 16),
            (14, 3, 16),
            (14, 4, 16),
        ] {
            place(
                &mut world,
                IVec3::new(x, y, z),
                BlockKind::Platform,
                Facing::North,
            );
        }
        let west_id = place(&mut world, west, BlockKind::Blocker, Facing::West);
        let east_id = place(&mut world, east, BlockKind::Blocker, Facing::East);

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sides_w = state.deform_sides(&world, west).unwrap();
        let sides_e = state.deform_sides(&world, east).unwrap();
        assert!(sides_w.separated, "west must separate after all-cut");
        assert!(sides_e.separated, "east must separate after all-cut");
        assert_eq!(
            sides_w.target_side, sides_e.actor_side,
            "west forward == east reverse"
        );
        assert_eq!(
            sides_w.actor_side, sides_e.target_side,
            "west reverse == east forward"
        );

        let structure = state.get(state.structure_id_at(west).unwrap()).unwrap();
        let w_fwd = structure.action_to_groups.get(&(west_id, true)).unwrap()[0];
        let e_rev = structure.action_to_groups.get(&(east_id, false)).unwrap()[0];
        assert_eq!(w_fwd, e_rev);
    }

    /// 伸出真实头后工厂连通仍穿过头
    #[test]
    fn extended_pusher_head_keeps_front_connected() {
        let mut world = WorldBlocks::default();
        let body = IVec3::new(0, 1, 0);
        let head = IVec3::new(1, 1, 0);
        let front = IVec3::new(2, 1, 0);
        world.insert(body, BlockData::new(BlockKind::Blocker, Facing::East));
        world.insert(head, BlockData::new(BlockKind::PusherHead, Facing::East));
        world.insert(front, BlockData::new(BlockKind::Platform, Facing::North));

        let members = factory_structure(&world, body);
        assert!(members.contains(&front));
        assert!(!members.contains(&head));

        let mut state = StructureState::default();
        state.rebuild_for_simulation(&world);
        let sides = state.deform_sides(&world, body).expect("sides");
        assert!(sides.separated);
        assert!(sides.target_side.contains(&front));
        assert!(!sides.target_side.contains(&body));
    }
