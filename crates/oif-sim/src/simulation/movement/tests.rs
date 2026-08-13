use super::*;
use crate::blocks::{BlockData, BlockKind, SceneBlockId};
use crate::simulation::structure_state::StructureState;
use crate::simulation::structures::{
    MovementInfluenceCache, MovementMark, StructureMove, execute_structure_moves_with_pushers,
};
use crate::simulation::suction::SuctionLinks;
use crate::world::Facing;
use glam::IVec3;
use std::collections::HashSet;

/// 准星读到的面对面活塞：北向 Pusher@(11,2,-4) 对南向 Pusher@(11,2,-5)
fn face_to_face_cursor_world() -> (WorldBlocks, IVec3, IVec3) {
    let mut world = WorldBlocks::default();
    // 地面（现场 Scene）
    for x in 9..13 {
        for z in -6..2 {
            world.insert(
                IVec3::new(x, 0, z),
                BlockData::new(BlockKind::Scene(SceneBlockId(6)), Facing::North),
            );
        }
    }
    let north = IVec3::new(11, 2, -4);
    let south = IVec3::new(11, 2, -5);
    world.insert(south, BlockData::new(BlockKind::Pusher, Facing::South));
    world.insert(north, BlockData::new(BlockKind::Pusher, Facing::North));
    world.insert(
        IVec3::new(11, 2, -3),
        BlockData::new(BlockKind::Wire, Facing::North),
    );
    world.insert(
        IVec3::new(11, 3, -3),
        BlockData::new(BlockKind::Detector, Facing::South),
    );
    world.insert(
        IVec3::new(11, 4, -2),
        BlockData::new(BlockKind::Platform, Facing::North),
    );
    (world, north, south)
}

fn run_pusher_phase(
    world: &mut WorldBlocks,
    structures: &mut StructureState,
    pusher_state: &mut PusherState,
    powered: &HashSet<IVec3>,
) {
    let suction = SuctionLinks::rebuild(world, structures, powered);
    let (moves, _) =
        mark_structure_movement_phase(world, powered, structures, pusher_state, &suction);
    let heads = PusherState::hard_head_occupancy(world);
    let mut influence = MovementInfluenceCache::default();
    let (_a, _p, commits) = execute_structure_moves_with_pushers(
        world,
        moves,
        structures,
        &mut influence,
        &heads,
        &suction,
    );
    for (id, (pos, extended)) in commits {
        pusher_state.set_extended(world, id, pos, extended);
    }
}

/// Blocker+Pusher 共轴：未通电时仅 Blocker 欲伸，不得单独伸出或反推撕裂
#[test]
fn coaxial_blocker_pusher_idle_power_stays_coupled() {
    let mut world = WorldBlocks::default();
    world.insert(
        IVec3::new(-4, 2, -6),
        BlockData::new(BlockKind::Platform, Facing::East),
    );
    world.insert(
        IVec3::new(-3, 2, -6),
        BlockData::new(BlockKind::Blocker, Facing::East),
    );
    world.insert(
        IVec3::new(-2, 2, -6),
        BlockData::new(BlockKind::Platform, Facing::North),
    );
    world.insert(
        IVec3::new(-4, 2, -5),
        BlockData::new(BlockKind::Platform, Facing::East),
    );
    world.insert(
        IVec3::new(-2, 2, -5),
        BlockData::new(BlockKind::Platform, Facing::North),
    );
    world.insert(
        IVec3::new(-4, 2, -4),
        BlockData::new(BlockKind::Platform, Facing::East),
    );
    world.insert(
        IVec3::new(-3, 2, -4),
        BlockData::new(BlockKind::Pusher, Facing::East),
    );
    world.insert(
        IVec3::new(-2, 2, -4),
        BlockData::new(BlockKind::Platform, Facing::North),
    );
    let blocker_pos = IVec3::new(-3, 2, -6);
    let pusher_pos = IVec3::new(-3, 2, -4);
    let blocker_id = world.blocks.get(&blocker_pos).unwrap().id;
    let pusher_id = world.blocks.get(&pusher_pos).unwrap().id;

    let mut structures = StructureState::default();
    structures.rebuild_for_simulation(&world);
    let mut pusher_state = PusherState::rebuild_from_world(&world);
    run_pusher_phase(
        &mut world,
        &mut structures,
        &mut pusher_state,
        &HashSet::new(),
    );
    structures.rebuild_for_simulation(&world);

    assert_eq!(
        world.blocks.get(&blocker_pos).map(|b| b.id),
        Some(blocker_id)
    );
    assert_eq!(world.blocks.get(&pusher_pos).map(|b| b.id), Some(pusher_id));
    assert!(
        !pusher_state
            .entries
            .get(&blocker_id)
            .is_some_and(|e| e.extended)
    );
    assert!(
        !pusher_state
            .entries
            .get(&pusher_id)
            .is_some_and(|e| e.extended)
    );
    assert_eq!(structures.id_at(blocker_pos), structures.id_at(pusher_pos));
    let st = structures
        .get(structures.id_at(blocker_pos).unwrap())
        .unwrap();
    assert_eq!(st.deform_groups.len(), 2);
}

/// 正推顶场景失败后反推一次并进入伸出，次回合不再后退
#[test]
fn scene_blocked_forward_reverse_extend_once() {
    let mut world = WorldBlocks::default();
    let body = IVec3::new(9, 2, 8);
    let front = IVec3::new(9, 2, 7);
    // 场景挡在货物北侧
    world.insert(
        IVec3::new(9, 2, 6),
        BlockData::new(BlockKind::Scene(SceneBlockId(6)), Facing::North),
    );
    world.insert(front, BlockData::new(BlockKind::Platform, Facing::North));
    world.insert(body, BlockData::new(BlockKind::Blocker, Facing::North));
    let id = world.blocks.get(&body).unwrap().id;

    let mut structures = StructureState::default();
    structures.rebuild_for_simulation(&world);
    let mut pusher_state = PusherState::rebuild_from_world(&world);
    run_pusher_phase(
        &mut world,
        &mut structures,
        &mut pusher_state,
        &HashSet::new(),
    );

    assert!(
        pusher_state.entries.get(&id).is_some_and(|e| e.extended),
        "should reverse-then-extend"
    );
    let body_after = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == id)
        .map(|(p, _)| *p)
        .unwrap();
    assert_eq!(body_after, IVec3::new(9, 2, 9), "body nudges south once");

    // 第二回合不应再退
    structures.rebuild_for_simulation(&world);
    run_pusher_phase(
        &mut world,
        &mut structures,
        &mut pusher_state,
        &HashSet::new(),
    );
    let body_2 = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == id)
        .map(|(p, _)| *p)
        .unwrap();
    assert_eq!(body_2, body_after, "must not keep reversing each turn");
}

/// 平行双杆正面列相连：仅 Blocker 欲伸出时须拖走未动的 Pusher 本体
#[test]
fn parallel_gap_solo_blocker_extend_drags_pusher_body() {
    let mut world = WorldBlocks::default();
    let pusher_pos = IVec3::new(-4, 2, -5);
    let blocker_pos = IVec3::new(-4, 2, -3);
    world.insert(pusher_pos, BlockData::new(BlockKind::Pusher, Facing::East));
    world.insert(
        blocker_pos,
        BlockData::new(BlockKind::Blocker, Facing::East),
    );
    world.insert(
        IVec3::new(-3, 2, -5),
        BlockData::new(BlockKind::Platform, Facing::North),
    );
    world.insert(
        IVec3::new(-3, 2, -4),
        BlockData::new(BlockKind::Platform, Facing::North),
    );
    world.insert(
        IVec3::new(-3, 2, -3),
        BlockData::new(BlockKind::Platform, Facing::North),
    );
    let pusher_id = world.blocks.get(&pusher_pos).unwrap().id;

    let mut structures = StructureState::default();
    structures.rebuild_for_simulation(&world);
    let mut pusher_state = PusherState::rebuild_from_world(&world);
    let powered = HashSet::new();
    run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);

    let pusher_after = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == pusher_id)
        .map(|(p, _)| *p)
        .expect("pusher");
    assert_eq!(
        pusher_after,
        pusher_pos + IVec3::new(1, 0, 0),
        "solo Blocker extend must drag Pusher body east; was {pusher_after:?}"
    );
}

#[test]
fn face_to_face_pusher_extend_retract_body_stays() {
    let (mut world, north, south) = face_to_face_cursor_world();
    let mut structures = StructureState::default();
    structures.rebuild_for_simulation(&world);
    let mut pusher_state = PusherState::rebuild_from_world(&world);

    let north_id = world.blocks.get(&north).unwrap().id;
    let south_id = world.blocks.get(&south).unwrap().id;

    // 仅通电北向杆一回合（对面无电）
    let powered = HashSet::from([north]);
    run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);

    let north_after_ext = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == north_id)
        .map(|(p, _)| *p);

    assert_eq!(
        north_after_ext,
        Some(north),
        "extend must not move the powered pusher body"
    );

    // 断电收回
    let powered = HashSet::new();
    run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);

    let north_after_ret = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == north_id)
        .map(|(p, _)| *p);
    let south_after_ret = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == south_id)
        .map(|(p, _)| *p);

    assert_eq!(
        north_after_ret,
        Some(north),
        "retract must not move the powered pusher body"
    );
    assert_eq!(
        south_after_ret,
        Some(south),
        "opposite pusher should return to original cell after retract"
    );
}

/// 三连同向 Blocker 链：每回合最多再伸一根，后手不得把头伸进仍停在身前的同伴
#[test]
fn same_facing_blocker_chain_no_head_body_overlap() {
    let mut world = WorldBlocks::default();
    let front = IVec3::new(4, 1, -14);
    let mid = IVec3::new(4, 1, -13);
    let rear = IVec3::new(4, 1, -12);
    world.insert(front, BlockData::new(BlockKind::Blocker, Facing::North));
    world.insert(mid, BlockData::new(BlockKind::Blocker, Facing::North));
    world.insert(rear, BlockData::new(BlockKind::Blocker, Facing::North));
    let ids = [
        world.blocks.get(&front).unwrap().id,
        world.blocks.get(&mid).unwrap().id,
        world.blocks.get(&rear).unwrap().id,
    ];
    let mut structures = StructureState::default();
    structures.rebuild_for_simulation(&world);
    let mut pusher_state = PusherState::rebuild_from_world(&world);
    let powered = HashSet::new();
    for turn in 1..=3 {
        run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);
        structures.rebuild_for_simulation(&world);
        let extended_count = ids
            .iter()
            .filter(|id| pusher_state.entries.get(id).is_some_and(|e| e.extended))
            .count();
        let expected = match turn {
            1 => 2, // 前空头 + 中反推
            _ => 3,
        };
        assert_eq!(
            extended_count, expected,
            "turn {turn}: deform chain extended={extended_count} expected={expected}"
        );
        for id in ids {
            let Some(body) = world
                .blocks
                .iter()
                .find(|(_, b)| b.id == id)
                .map(|(p, _)| *p)
            else {
                continue;
            };
            let ext = pusher_state.entries.get(&id).is_some_and(|e| e.extended);
            let facing = world.blocks.get(&body).unwrap().facing;
            let head = body + facing.forward_ivec3();
            if ext {
                assert_eq!(
                    world.blocks.get(&head).map(|b| b.kind),
                    Some(BlockKind::PusherHead),
                    "turn {turn}: extended {id:?} at {body:?} needs real head"
                );
            } else if world
                .blocks
                .get(&head)
                .is_some_and(|f| matches!(f.kind, BlockKind::Blocker | BlockKind::Pusher))
            {
                // 身前仍是同伴本体：保持 hold
            }
        }
        for (pos, b) in &world.blocks {
            if !matches!(b.kind, BlockKind::Blocker | BlockKind::Pusher) {
                continue;
            }
            let head = *pos + b.facing.forward_ivec3();
            if world
                .blocks
                .get(&head)
                .is_some_and(|f| matches!(f.kind, BlockKind::Blocker | BlockKind::Pusher))
            {
                assert!(
                    !pusher_state.entries.get(&b.id).is_some_and(|e| e.extended),
                    "turn {turn}: blocker at {pos:?} must not extend into peer at {head:?}"
                );
            }
        }
    }
}

/// 准星贴脸对向：北 Blocker@(0,2,-14) 对南 Blocker@(0,2,-15)
/// 先手正推后，后手本回合 hold，不可再伸头（否则两头叠在中间格）
fn face_to_face_blocker_pair_world() -> (WorldBlocks, IVec3, IVec3) {
    let mut world = WorldBlocks::default();
    let north = IVec3::new(0, 2, -14);
    let south = IVec3::new(0, 2, -15);
    world.insert(south, BlockData::new(BlockKind::Blocker, Facing::South));
    world.insert(north, BlockData::new(BlockKind::Blocker, Facing::North));
    (world, north, south)
}

#[test]
fn face_to_face_blocker_deform_actions() {
    let (mut world, north, south) = face_to_face_blocker_pair_world();
    let mut structures = StructureState::default();
    structures.rebuild_for_simulation(&world);
    let mut pusher_state = PusherState::rebuild_from_world(&world);
    let north_id = world.blocks.get(&north).unwrap().id;
    let south_id = world.blocks.get(&south).unwrap().id;
    assert!(south_id.0 < north_id.0);

    let powered = HashSet::new();
    run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);
    structures.rebuild_for_simulation(&world);

    assert!(
        pusher_state
            .entries
            .get(&south_id)
            .is_some_and(|e| e.extended)
    );
    assert!(
        !pusher_state
            .entries
            .get(&north_id)
            .is_some_and(|e| e.extended)
    );

    let heads: Vec<_> = world
        .blocks
        .iter()
        .filter(|(_, b)| b.kind == BlockKind::PusherHead)
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(heads.len(), 1);

    let north_pos = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == north_id)
        .map(|(p, _)| *p)
        .unwrap();
    let south_pos = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == south_id)
        .map(|(p, _)| *p)
        .unwrap();
    assert_eq!(south_pos, south);
    assert_eq!(north_pos, north + IVec3::new(0, 0, 1));
    assert_eq!(structures.id_at(north_pos), structures.id_at(south_pos));
}

#[test]
fn bent_three_blocker_east_forward_north_reverse() {
    let mut world = WorldBlocks::default();
    let east = IVec3::new(8, 2, -13);
    let south = IVec3::new(9, 2, -13);
    let north = IVec3::new(8, 2, -12);
    world.insert(east, BlockData::new(BlockKind::Blocker, Facing::East));
    world.insert(south, BlockData::new(BlockKind::Blocker, Facing::South));
    world.insert(north, BlockData::new(BlockKind::Blocker, Facing::North));

    let mut structures = StructureState::default();
    structures.rebuild_for_simulation(&world);
    let mut pusher_state = PusherState::rebuild_from_world(&world);
    let east_id = world.blocks.get(&east).unwrap().id;
    let south_id = world.blocks.get(&south).unwrap().id;
    let north_id = world.blocks.get(&north).unwrap().id;
    assert!(east_id.0 < south_id.0 && south_id.0 < north_id.0);

    let powered = HashSet::new();
    run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);

    assert!(
        pusher_state
            .entries
            .get(&east_id)
            .is_some_and(|e| e.extended)
    );
    assert!(
        !pusher_state
            .entries
            .get(&south_id)
            .is_some_and(|e| e.extended)
    );
    assert!(
        pusher_state
            .entries
            .get(&north_id)
            .is_some_and(|e| e.extended)
    );

    let east_pos = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == east_id)
        .map(|(p, _)| *p)
        .unwrap();
    let south_pos = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == south_id)
        .map(|(p, _)| *p)
        .unwrap();
    let north_pos = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == north_id)
        .map(|(p, _)| *p)
        .unwrap();
    assert_eq!(east_pos, east);
    assert_eq!(south_pos, south + IVec3::X);
    assert_eq!(north_pos, north + IVec3::new(0, 0, 1));
}

/// 准星 2×2 同向四格环：两西向 Blocker + 身前 Platform，伸出后仍是一块工厂结构
fn same_facing_blocker_square_world() -> (WorldBlocks, IVec3, IVec3) {
    let mut world = WorldBlocks::default();
    let a = IVec3::new(-12, 2, -13);
    let b = IVec3::new(-12, 2, -12);
    world.insert(
        IVec3::new(-13, 2, -13),
        BlockData::new(BlockKind::Platform, Facing::West),
    );
    world.insert(
        IVec3::new(-13, 2, -12),
        BlockData::new(BlockKind::Platform, Facing::West),
    );
    world.insert(a, BlockData::new(BlockKind::Blocker, Facing::West));
    world.insert(b, BlockData::new(BlockKind::Blocker, Facing::West));
    (world, a, b)
}

#[test]
fn same_facing_blocker_square_extend_stays_one_structure() {
    let (mut world, a, b) = same_facing_blocker_square_world();
    let mut structures = StructureState::default();
    structures.rebuild_for_simulation(&world);
    let mut pusher_state = PusherState::rebuild_from_world(&world);
    let a_id = world.blocks.get(&a).unwrap().id;
    let b_id = world.blocks.get(&b).unwrap().id;
    assert_eq!(structures.id_at(a), structures.id_at(b));
    assert_eq!(
        structures
            .structure_positions(structures.id_at(a).unwrap())
            .map(|s| s.len()),
        Some(4)
    );

    let powered = HashSet::new();
    run_pusher_phase(&mut world, &mut structures, &mut pusher_state, &powered);
    structures.rebuild_for_simulation(&world);

    let a_pos = world
        .blocks
        .iter()
        .find(|(_, blk)| blk.id == a_id)
        .map(|(p, _)| *p)
        .expect("blocker a");
    let b_pos = world
        .blocks
        .iter()
        .find(|(_, blk)| blk.id == b_id)
        .map(|(p, _)| *p)
        .expect("blocker b");
    assert_eq!(
        a_pos, a,
        "same-facing square must not reverse-drag blocker bodies"
    );
    assert_eq!(
        b_pos, b,
        "same-facing square must not reverse-drag blocker bodies"
    );
    assert_eq!(
        structures.id_at(a_pos),
        structures.id_at(b_pos),
        "factory structure must stay one piece after extend"
    );
    assert_eq!(
        structures
            .structure_positions(structures.id_at(a_pos).unwrap())
            .map(|s| s.len()),
        Some(4),
        "still four factory members (heads are not members)"
    );
    for (id, body) in [(a_id, a_pos), (b_id, b_pos)] {
        let facing = world.blocks.get(&body).unwrap().facing;
        let head = body + facing.forward_ivec3();
        assert_eq!(
            world.blocks.get(&head).map(|blk| blk.kind),
            Some(BlockKind::PusherHead),
            "blocker {id:?} needs its own head cell"
        );
    }
}

/// 准星 8 格 3×3 对向拦截器环：一正推应拖走同伴，两杆错开竖轴且头不叠块
fn opposing_blocker_ring_world() -> (WorldBlocks, IVec3, IVec3) {
    let mut world = WorldBlocks::default();
    let west = IVec3::new(13, 2, 16);
    let east = IVec3::new(13, 4, 16);
    // 3×3 环：两侧立柱平台 + 对向 Blocker，中心空
    for (x, y, z) in [
        (12, 2, 16),
        (12, 3, 16),
        (12, 4, 16),
        (14, 2, 16),
        (14, 3, 16),
        (14, 4, 16),
    ] {
        world.insert(
            IVec3::new(x, y, z),
            BlockData::new(BlockKind::Platform, Facing::North),
        );
    }
    world.insert(west, BlockData::new(BlockKind::Blocker, Facing::West));
    world.insert(east, BlockData::new(BlockKind::Blocker, Facing::East));
    (world, west, east)
}

#[test]
fn opposing_blocker_ring_one_push_one_reverse_keeps_one_structure() {
    let (mut world, west, east) = opposing_blocker_ring_world();
    let mut structures = StructureState::default();
    structures.rebuild_for_simulation(&world);
    let mut pusher_state = PusherState::rebuild_from_world(&world);

    let west_id = world.blocks.get(&west).unwrap().id;
    let east_id = world.blocks.get(&east).unwrap().id;
    let sid_before = structures.id_at(west).expect("structure");
    assert_eq!(structures.id_at(east), Some(sid_before));
    assert_eq!(
        structures.structure_positions(sid_before).map(|s| s.len()),
        Some(8)
    );

    // Blocker：断电要伸出
    let powered = HashSet::new();
    let suction = SuctionLinks::rebuild(&world, &structures, &powered);
    let moves = mark_structure_movement_phase(
        &mut world,
        &powered,
        &mut structures,
        &mut pusher_state,
        &suction,
    )
    .0;

    let mut nonzero_push_dirs = Vec::new();
    let mut actor_anims = Vec::new();
    for m in &moves {
        if let StructureMove::Translate {
            offset,
            actors,
            mark: MovementMark::Push,
            structure,
            ..
        } = m
        {
            for a in actors {
                actor_anims.push((a.id, a.pos, *offset, structure.len()));
            }
            if *offset != IVec3::ZERO {
                nonzero_push_dirs.push(*offset);
            }
        }
    }
    assert!(
        !nonzero_push_dirs.is_empty() && !actor_anims.is_empty(),
        "ring should move; dirs={nonzero_push_dirs:?} actors={actor_anims:?}"
    );

    let heads = PusherState::hard_head_occupancy(&world);
    let mut influence = MovementInfluenceCache::default();
    let (_a, _p, commits) = execute_structure_moves_with_pushers(
        &mut world,
        moves,
        &mut structures,
        &mut influence,
        &heads,
        &suction,
    );
    for (id, (pos, extended)) in commits {
        pusher_state.set_extended(&mut world, id, pos, extended);
    }
    structures.rebuild_for_simulation(&world);

    let west_pos = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == west_id)
        .map(|(p, _)| *p)
        .expect("west blocker");
    let east_pos = world
        .blocks
        .iter()
        .find(|(_, b)| b.id == east_id)
        .map(|(p, _)| *p)
        .expect("east blocker");
    assert_ne!(
        west_pos.x, east_pos.x,
        "reverse/drag must unstack blockers off the same vertical axis: west={west_pos:?} east={east_pos:?}"
    );
    assert_eq!(
        structures.id_at(west_pos),
        structures.id_at(east_pos),
        "ring must stay one structure after the turn"
    );
    // 两杆都应伸出真实头，且头格不得被其它方块占用（HashMap 单格单块；缺头即曾与身前重叠而未写入）
    for (id, body) in [(west_id, west_pos), (east_id, east_pos)] {
        let facing = world.blocks.get(&body).unwrap().facing;
        let head = body + facing.forward_ivec3();
        let head_kind = world.blocks.get(&head).map(|b| b.kind);
        assert_eq!(
            head_kind,
            Some(BlockKind::PusherHead),
            "blocker {id:?} at {body:?} must own empty head cell {head:?}, got {head_kind:?}"
        );
        assert!(
            pusher_state.entries.get(&id).is_some_and(|e| e.extended),
            "blocker {id:?} should be extended"
        );
    }
}

/// 面对面：南小 ID 先手应 198+；北小 ID 先手应 5+；次回合后手应正推顶头而非反推
#[test]
fn face_pair_south_lower_id_and_second_turn_forward() {
    use crate::simulation::core::simulate_turn;
    use crate::simulation::pending::PendingGeneratedMaterials;
    use crate::simulation::signals::SignalNetworkCache;
    use crate::simulation::structures::MovementInfluenceCache;

    fn floor(world: &mut WorldBlocks, x0: i32, x1: i32, z0: i32, z1: i32) {
        for z in z0..z1 {
            for x in x0..x1 {
                world.insert(
                    IVec3::new(x, 0, z),
                    BlockData::new(BlockKind::Scene(SceneBlockId(6)), Facing::North),
                );
            }
        }
    }

    fn pos_of(world: &WorldBlocks, id: crate::blocks::BlockId) -> IVec3 {
        world
            .blocks
            .iter()
            .find(|(_, b)| b.id == id)
            .map(|(p, _)| *p)
            .expect("block id present")
    }

    // Pair A: 北小 ID（同 #5/#156）
    {
        let mut world = WorldBlocks::default();
        floor(&mut world, -1, 2, -20, -10);
        let north = IVec3::new(0, 2, -14);
        let south = IVec3::new(0, 2, -15);
        world.insert(north, BlockData::new(BlockKind::Blocker, Facing::North));
        world.insert(south, BlockData::new(BlockKind::Blocker, Facing::South));
        let n_id = world.blocks.get(&north).unwrap().id;
        let s_id = world.blocks.get(&south).unwrap().id;
        assert!(n_id.0 < s_id.0);
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher = PusherState::rebuild_from_world(&world);
        let mut pending = PendingGeneratedMaterials::default();
        let mut signals = SignalNetworkCache::default();
        let mut influence = MovementInfluenceCache::default();
        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            1,
            &mut structures,
            &mut influence,
            &mut pusher,
            None,
            None,
        );
        assert!(
            pusher.entries.get(&n_id).is_some_and(|e| e.extended),
            "T1 north+"
        );
        assert!(
            !pusher.entries.get(&s_id).is_some_and(|e| e.extended),
            "T1 south held"
        );
        assert_eq!(pos_of(&world, n_id), north);
        assert_eq!(pos_of(&world, s_id), south + IVec3::NEG_Z);
        let head = world.blocks.get(&(north + IVec3::NEG_Z));
        assert_eq!(head.map(|b| b.kind), Some(BlockKind::PusherHead));
        assert_eq!(head.map(|b| b.facing), Some(Facing::North));
        // T2: south 正推顶头，北侧被顶走；反推会把南体再往 -Z 移一格
        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            2,
            &mut structures,
            &mut influence,
            &mut pusher,
            None,
            None,
        );
        assert!(
            pusher.entries.get(&s_id).is_some_and(|e| e.extended),
            "T2 south should forward-extend"
        );
        assert_eq!(
            pos_of(&world, s_id),
            south + IVec3::NEG_Z,
            "T2 must not reverse south"
        );
        assert_eq!(
            pos_of(&world, n_id),
            north + IVec3::Z,
            "T2 forward pushes north partner"
        );
        let hs = world.blocks.get(&(south + IVec3::NEG_Z + IVec3::Z));
        assert_eq!(
            hs.map(|b| b.facing),
            Some(Facing::South),
            "T2 south head facing"
        );
    }

    // Pair B: 南小 ID（同 #198/#199）
    {
        let mut world = WorldBlocks::default();
        floor(&mut world, 8, 11, -8, 0);
        let south = IVec3::new(9, 2, -5);
        let north = IVec3::new(9, 2, -4);
        world.insert(south, BlockData::new(BlockKind::Blocker, Facing::South));
        world.insert(north, BlockData::new(BlockKind::Blocker, Facing::North));
        let s_id = world.blocks.get(&south).unwrap().id;
        let n_id = world.blocks.get(&north).unwrap().id;
        assert!(s_id.0 < n_id.0);
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher = PusherState::rebuild_from_world(&world);
        let mut pending = PendingGeneratedMaterials::default();
        let mut signals = SignalNetworkCache::default();
        let mut influence = MovementInfluenceCache::default();
        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            1,
            &mut structures,
            &mut influence,
            &mut pusher,
            None,
            None,
        );
        assert!(
            pusher.entries.get(&s_id).is_some_and(|e| e.extended),
            "T1 south lower-id must 198+"
        );
        assert!(
            !pusher.entries.get(&n_id).is_some_and(|e| e.extended),
            "T1 north must not 199-"
        );
        assert_eq!(pos_of(&world, s_id), south);
        assert_eq!(pos_of(&world, n_id), north + IVec3::Z);
        let head = world.blocks.get(&(south + IVec3::Z));
        assert_eq!(head.map(|b| b.facing), Some(Facing::South));
        // T2: north 正推顶头
        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            2,
            &mut structures,
            &mut influence,
            &mut pusher,
            None,
            None,
        );
        assert!(
            pusher.entries.get(&n_id).is_some_and(|e| e.extended),
            "T2 north should forward-extend"
        );
        assert_eq!(
            pos_of(&world, n_id),
            north + IVec3::Z,
            "T2 must not reverse north"
        );
        assert_eq!(
            pos_of(&world, s_id),
            south + IVec3::NEG_Z,
            "T2 forward pushes south partner"
        );
        let hn = world.blocks.get(&(north + IVec3::Z + IVec3::NEG_Z));
        assert_eq!(
            hn.map(|b| b.facing),
            Some(Facing::North),
            "T2 north head facing"
        );
    }
}

/// 移动占位：⬛ 均为独立材料结构（无焊接）
mod moving_occupancy_cases {
    use super::*;
    use crate::blocks::BlockKind;
    use crate::simulation::core::simulate_turn;
    use crate::simulation::pending::PendingGeneratedMaterials;
    use crate::simulation::signals::SignalNetworkCache;
    use crate::simulation::structures::MovementInfluenceCache;

    fn mat() -> BlockData {
        BlockData::new(BlockKind::material("iron"), Facing::North)
    }

    fn scene() -> BlockData {
        BlockData::new(BlockKind::Scene(SceneBlockId(6)), Facing::North)
    }

    fn turn(
        world: &mut WorldBlocks,
        structures: &mut StructureState,
        pusher: &mut PusherState,
        n: u64,
    ) {
        let mut pending = PendingGeneratedMaterials::default();
        let mut signals = SignalNetworkCache::default();
        let mut influence = MovementInfluenceCache::default();
        simulate_turn(
            world,
            &mut pending,
            &mut signals,
            n,
            structures,
            &mut influence,
            pusher,
            None,
            None,
        );
    }

    fn id_at(world: &WorldBlocks, pos: IVec3) -> crate::blocks::BlockId {
        world.blocks.get(&pos).expect("block at pos").id
    }

    fn pos_of(world: &WorldBlocks, id: crate::blocks::BlockId) -> IVec3 {
        world
            .blocks
            .iter()
            .find(|(_, b)| b.id == id)
            .map(|(p, _)| *p)
            .expect("id present")
    }

    /// 右块悬空下落占双格 → 左块本回合不能右移（两 ⬛ 独立）
    #[test]
    fn fall_occupies_start_blocks_lateral_conveyor() {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::new(0, 0, 0), scene());
        world.insert(
            IVec3::new(0, 1, 0),
            BlockData::new(BlockKind::Conveyor, Facing::East),
        );
        world.insert(IVec3::new(0, 2, 0), mat());
        world.insert(IVec3::new(1, 2, 0), mat());
        let left = id_at(&world, IVec3::new(0, 2, 0));
        let right = id_at(&world, IVec3::new(1, 2, 0));
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher = PusherState::rebuild_from_world(&world);
        turn(&mut world, &mut structures, &mut pusher, 1);
        assert_eq!(pos_of(&world, right), IVec3::new(1, 1, 0), "right falls");
        assert_eq!(
            pos_of(&world, left),
            IVec3::new(0, 2, 0),
            "left must not enter falling cell"
        );
    }

    /// 双传送带同向：两独立块同回合都右移
    #[test]
    fn convoy_same_direction_both_move() {
        let mut world = WorldBlocks::default();
        for x in 0..3 {
            world.insert(IVec3::new(x, 0, 0), scene());
        }
        world.insert(
            IVec3::new(0, 1, 0),
            BlockData::new(BlockKind::Conveyor, Facing::East),
        );
        world.insert(
            IVec3::new(1, 1, 0),
            BlockData::new(BlockKind::Conveyor, Facing::East),
        );
        world.insert(IVec3::new(0, 2, 0), mat());
        world.insert(IVec3::new(1, 2, 0), mat());
        let left = id_at(&world, IVec3::new(0, 2, 0));
        let right = id_at(&world, IVec3::new(1, 2, 0));
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher = PusherState::rebuild_from_world(&world);
        turn(&mut world, &mut structures, &mut pusher, 1);
        assert_eq!(pos_of(&world, left), IVec3::new(1, 2, 0));
        assert_eq!(pos_of(&world, right), IVec3::new(2, 2, 0));
    }

    /// 竖叠两独立块同向下落
    #[test]
    fn stacked_fall_same_direction() {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::new(0, 1, 0), mat());
        world.insert(IVec3::new(0, 2, 0), mat());
        let lower = id_at(&world, IVec3::new(0, 1, 0));
        let upper = id_at(&world, IVec3::new(0, 2, 0));
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher = PusherState::rebuild_from_world(&world);
        turn(&mut world, &mut structures, &mut pusher, 1);
        assert_eq!(pos_of(&world, lower), IVec3::new(0, 0, 0));
        assert_eq!(pos_of(&world, upper), IVec3::new(0, 1, 0));
    }

    /// 悬空先落 → 上带右运 → 下带承接
    #[test]
    fn chain_fall_then_east_then_west() {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::new(0, 0, 0), scene());
        world.insert(IVec3::new(2, 0, 0), scene());
        world.insert(
            IVec3::new(0, 1, 0),
            BlockData::new(BlockKind::Conveyor, Facing::East),
        );
        // 西向带在下落落点下方：材料停在 (1,1)，带在 (1,0)
        world.insert(
            IVec3::new(1, 0, 0),
            BlockData::new(BlockKind::Conveyor, Facing::East),
        );
        world.insert(IVec3::new(2, 0, 0), scene());
        world.insert(IVec3::new(0, 2, 0), mat());
        world.insert(IVec3::new(1, 2, 0), mat());
        let left = id_at(&world, IVec3::new(0, 2, 0));
        let right = id_at(&world, IVec3::new(1, 2, 0));
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher = PusherState::rebuild_from_world(&world);

        turn(&mut world, &mut structures, &mut pusher, 1);
        assert_eq!(pos_of(&world, right), IVec3::new(1, 1, 0), "T1 fall");
        assert_eq!(pos_of(&world, left), IVec3::new(0, 2, 0), "T1 left held");

        turn(&mut world, &mut structures, &mut pusher, 2);
        assert_eq!(
            pos_of(&world, left),
            IVec3::new(1, 2, 0),
            "T2 upper belt east"
        );

        turn(&mut world, &mut structures, &mut pusher, 3);
        assert_eq!(
            pos_of(&world, right),
            IVec3::new(2, 1, 0),
            "T3 lower belt east"
        );
    }

    /// 上块曾撑在 Active 下块上：下块本回合可落时空时，上块不得因跨回合 support 拒落
    #[test]
    fn support_on_active_must_not_block_cofall() {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::new(0, 0, 0), scene());
        world.insert(IVec3::new(0, 1, 0), mat());
        world.insert(IVec3::new(0, 3, 0), mat());
        world.insert(IVec3::new(0, 4, 0), mat());
        let low = id_at(&world, IVec3::new(0, 1, 0));
        let mid = id_at(&world, IVec3::new(0, 3, 0));
        let top = id_at(&world, IVec3::new(0, 4, 0));

        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let top_sid = structures.id_at(IVec3::new(0, 4, 0)).unwrap();
        // 模拟上回合撑在 Active mid 上写入的缓存
        structures.record_gravity_support(top_sid, &world, &HashSet::new());
        assert!(
            !structures.gravity_support_valid(top_sid, &world, &HashSet::new()),
            "support on Active must not be valid across turns"
        );

        let mut pusher = PusherState::rebuild_from_world(&world);
        turn(&mut world, &mut structures, &mut pusher, 1);
        assert_eq!(
            pos_of(&world, mid),
            IVec3::new(0, 2, 0),
            "mid falls into gap"
        );
        assert_eq!(
            pos_of(&world, top),
            IVec3::new(0, 3, 0),
            "top cofalls with mid"
        );
        assert_eq!(pos_of(&world, low), IVec3::new(0, 1, 0), "low stays");
    }

    /// 对向活塞轮流：一伸一收同向经过中间格
    #[test]
    fn opposing_pistons_alternate_extend_retract() {
        let mut world = WorldBlocks::default();
        for x in 0..3 {
            world.insert(IVec3::new(x, 0, 0), scene());
        }
        let left = IVec3::new(0, 1, 0);
        let right = IVec3::new(2, 1, 0);
        world.insert(left, BlockData::new(BlockKind::Pusher, Facing::East));
        world.insert(right, BlockData::new(BlockKind::Pusher, Facing::West));
        let left_id = id_at(&world, left);
        let right_id = id_at(&world, right);
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let mut pusher = PusherState::rebuild_from_world(&world);

        run_pusher_phase(
            &mut world,
            &mut structures,
            &mut pusher,
            &HashSet::from([right]),
        );
        assert!(
            pusher.entries.get(&right_id).is_some_and(|e| e.extended),
            "T1 right extends"
        );
        assert!(
            !pusher.entries.get(&left_id).is_some_and(|e| e.extended),
            "T1 left retracted"
        );

        run_pusher_phase(
            &mut world,
            &mut structures,
            &mut pusher,
            &HashSet::from([left]),
        );
        assert!(
            pusher.entries.get(&left_id).is_some_and(|e| e.extended),
            "T2 left must extend while right retracts"
        );
        assert!(
            !pusher.entries.get(&right_id).is_some_and(|e| e.extended),
            "T2 right must retract"
        );

        run_pusher_phase(
            &mut world,
            &mut structures,
            &mut pusher,
            &HashSet::from([right]),
        );
        assert!(
            pusher.entries.get(&right_id).is_some_and(|e| e.extended),
            "T3 right extends again"
        );
        assert!(
            !pusher.entries.get(&left_id).is_some_and(|e| e.extended),
            "T3 left retracts"
        );
    }


    /// Active 结构高低交错支撑时，gravity grounded 不得死递归
    #[test]
    fn interleaved_active_support_cycle_does_not_hang() {
        // 材料 A 与工厂蛇 B 不相连；A 压在 B 上、B 又绕到 A 上方 → grounded 查询环
        let plat = || BlockData::new(BlockKind::Platform, Facing::North);
        let mat = || BlockData::new(BlockKind::Material(crate::blocks::MaterialBlockId(0)), Facing::North);
        let mut world = WorldBlocks::default();
        world.insert(IVec3::new(0, 2, 0), mat()); // A
        world.insert(IVec3::new(0, 1, 0), plat()); // B
        world.insert(IVec3::new(1, 1, 0), plat());
        world.insert(IVec3::new(1, 2, 0), plat());
        world.insert(IVec3::new(1, 3, 0), plat());
        world.insert(IVec3::new(0, 3, 0), plat());
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let a = structures.id_at(IVec3::new(0, 2, 0)).expect("A");
        let b = structures.id_at(IVec3::new(0, 1, 0)).expect("B");
        assert_ne!(a, b);
        assert_eq!(structures.id_at(IVec3::new(0, 3, 0)), Some(b));
        let mut pusher = PusherState::rebuild_from_world(&world);
        let mut pending = crate::simulation::pending::PendingGeneratedMaterials::default();
        let mut signals = crate::simulation::signals::SignalNetworkCache::default();
        let mut influence = MovementInfluenceCache::default();
        let _ = crate::simulation::core::simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            1,
            &mut structures,
            &mut influence,
            &mut pusher,
            None,
            None,
        );
    }

}
