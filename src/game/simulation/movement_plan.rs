use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::world::animation::{BlockAnimation, BlockAnimationKind, PusherAnimation};
use crate::game::world::grid::WorldBlocks;

use super::gravity::mark_gravity_phase;
use super::movement::{
    collect_conveyor_candidate, collect_lift_candidate, collect_powered_translate_candidate,
    collect_pusher_candidate, collect_rotate_candidate, PusherState,
};
use super::structure_state::StructureState;
use super::structures::{
    can_rotate_structure, expanded_move_structure, move_structure, rotate_structure,
    rotate_facing_internal, rotate_pos_y, MovementCandidate, MovementInfluenceCache,
    MovementMark, PusherAnimationKind, StructureMove, StructureMovePhaseKind,
};

#[derive(Clone)]
pub struct MovementPhasePlan {
    pub phase: StructureMovePhaseKind,
    pub candidates: Vec<MovementCandidate>,
}

#[derive(Clone, Default)]
pub struct MovementPlan {
    pub phases: Vec<MovementPhasePlan>,
    pub bare_pusher_animations: HashMap<IVec3, PusherAnimation>,
}

pub struct MovementExecutionOutput {
    pub animations: HashMap<IVec3, BlockAnimation>,
    pub pusher_animations: HashMap<IVec3, PusherAnimation>,
}

pub fn collect_movement_plan(
    turn: &WorldBlocks,
    solution: &WorldBlocks,
    turn_structures: &mut StructureState,
    solution_structures: &StructureState,
    powered_devices: &HashSet<IVec3>,
    pusher_state: &PusherState,
    movement_influence: &mut MovementInfluenceCache,
) -> MovementPlan {
    let mut rotate = Vec::new();
    let mut push = Vec::new();
    let mut lift = Vec::new();
    let mut conveyor = Vec::new();
    let mut bare_pusher_animations = HashMap::new();

    {
        let ctx = super::movement::MovementMarkContext {
            turn,
            solution,
            turn_structures,
            solution_structures,
        };
        for (pos, kind, rule) in super::movement::sorted_factory_movers(turn) {
            match rule {
                crate::game::blocks::MovementRule::Rotate { clockwise } => {
                    if let Some(candidate) =
                        collect_rotate_candidate(turn_structures, pos, clockwise)
                    {
                        rotate.push(candidate);
                    }
                }
                crate::game::blocks::MovementRule::PoweredTranslate { source, offset } => {
                    if matches!(
                        kind,
                        crate::game::blocks::BlockKind::Pusher | crate::game::blocks::BlockKind::Blocker
                    ) {
                        let desired_extended = if kind == crate::game::blocks::BlockKind::Pusher {
                            powered_devices.contains(&pos)
                        } else {
                            !powered_devices.contains(&pos)
                        };
                        let result = collect_pusher_candidate(
                            &ctx,
                            pos,
                            source,
                            offset,
                            desired_extended,
                            pusher_state,
                        );
                        if let Some(candidate) = result.candidate {
                            push.push(candidate);
                        }
                        if let Some((pusher_pos, animation)) = result.bare_animation {
                            bare_pusher_animations.insert(pusher_pos, animation);
                        }
                    } else if powered_devices.contains(&pos) {
                        if let Some(candidate) =
                            collect_powered_translate_candidate(&ctx, pos, source, offset)
                        {
                            push.push(candidate);
                        }
                    }
                }
                crate::game::blocks::MovementRule::Lift { range } => {
                    if let Some(candidate) = collect_lift_candidate(&ctx, pos, range) {
                        lift.push(candidate);
                    }
                }
                crate::game::blocks::MovementRule::Translate { source, offset } => {
                    if let Some(candidate) =
                        collect_conveyor_candidate(&ctx, pos, source, offset)
                    {
                        conveyor.push(candidate);
                    }
                }
            }
        }
    }

    movement_influence.begin_turn_from_candidates(&rotate, &conveyor);
    rotate.retain(|candidate| !movement_influence.ignores_rotator_movement(&candidate.primary));
    conveyor.sort_by(|a, b| movement_influence.compare_conveyor_candidates(a, b));
    movement_influence.finish_turn_from_candidates(&conveyor);

    let actuating = pusher_state.actuating_devices(turn, powered_devices);
    let hard_heads = pusher_state.hard_head_occupancy(turn);
    let gravity = mark_gravity_phase(turn, turn_structures, &actuating, &hard_heads)
        .into_iter()
        .map(|movement| MovementCandidate {
            primary: movement,
            fallbacks: Vec::new(),
        })
        .collect();

    let mut plan = MovementPlan::default();
    plan.phases.push(MovementPhasePlan {
        phase: StructureMovePhaseKind::Fixed,
        candidates: Vec::new(),
    });
    plan.phases.push(sorted_phase(StructureMovePhaseKind::Rotate, rotate));
    plan.phases.push(sorted_phase(StructureMovePhaseKind::Push, push));
    plan.phases.push(sorted_phase(StructureMovePhaseKind::Lift, lift));
    plan.phases.push(sorted_phase(StructureMovePhaseKind::Gravity, gravity));
    plan.phases.push(sorted_phase(StructureMovePhaseKind::Conveyor, conveyor));
    plan.bare_pusher_animations = bare_pusher_animations;
    plan
}

fn sorted_phase(phase: StructureMovePhaseKind, mut candidates: Vec<MovementCandidate>) -> MovementPhasePlan {
    candidates.sort_by_key(|candidate| source_sort_key(&candidate.primary));
    MovementPhasePlan { phase, candidates }
}

fn source_sort_key(movement: &StructureMove) -> (i32, i32, i32) {
    movement
        .source()
        .map(|pos| (pos.x, pos.y, pos.z))
        .unwrap_or((i32::MIN, i32::MIN, i32::MIN))
}

pub fn execute_movement_plan(
    plan: &MovementPlan,
    realtime: &mut WorldBlocks,
    turn_structures: &mut StructureState,
    pusher_state: &mut PusherState,
    movement_influence: &mut MovementInfluenceCache,
) -> MovementExecutionOutput {
    let mut moved = HashSet::new();
    let mut animations = HashMap::new();
    let mut pusher_animations = plan.bare_pusher_animations.clone();

    for phase in &plan.phases {
        if phase.phase == StructureMovePhaseKind::Push {
            for (&pos, animation) in &plan.bare_pusher_animations {
                if phase
                    .candidates
                    .iter()
                    .any(|candidate| candidate.primary.pusher_actor() == Some(pos))
                {
                    continue;
                }
                pusher_animations.entry(pos).or_insert(*animation);
            }
        }

        let hard_heads = pusher_state.hard_head_occupancy(realtime);
        for candidate in &phase.candidates {
            for movement in
                std::iter::once(&candidate.primary).chain(candidate.fallbacks.iter())
            {
                if try_execute_move(
                    realtime,
                    turn_structures,
                    pusher_state,
                    movement_influence,
                    &hard_heads,
                    movement,
                    &mut moved,
                    &mut animations,
                    &mut pusher_animations,
                ) {
                    break;
                }
            }
        }

        if phase.phase == StructureMovePhaseKind::Push {
            for (&pos, animation) in &plan.bare_pusher_animations {
                if pusher_animations.contains_key(&pos) {
                    apply_pusher_extension_from_animation(pusher_state, realtime, pos, animation);
                }
            }
        }
    }

    for (pos, animation) in pusher_state.sustained_animations() {
        pusher_animations.entry(pos).or_insert(animation);
    }

    MovementExecutionOutput {
        animations,
        pusher_animations,
    }
}

fn apply_pusher_extension_from_animation(
    pusher_state: &mut PusherState,
    world: &WorldBlocks,
    pos: IVec3,
    animation: &PusherAnimation,
) {
    let extended = animation.to_extension > animation.from_extension;
    pusher_state.set_extended(pos, world, extended);
}

fn try_execute_move(
    realtime: &mut WorldBlocks,
    turn_structures: &mut StructureState,
    pusher_state: &mut PusherState,
    movement_influence: &mut MovementInfluenceCache,
    hard_heads: &HashSet<IVec3>,
    movement: &StructureMove,
    moved: &mut HashSet<IVec3>,
    animations: &mut HashMap<IVec3, BlockAnimation>,
    pusher_animations: &mut HashMap<IVec3, PusherAnimation>,
) -> bool {
    match movement {
        StructureMove::Translate {
            structure,
            offset,
            actor,
            mark,
            source,
        } => {
            if structure.iter().any(|pos| moved.contains(pos)) {
                return false;
            }
            if blocks_gravity_pusher_head(realtime, structure, hard_heads, *offset) {
                return false;
            }
            let mode = super::structures::movement_expansion_mode_public(*mark, *source);
            let Some(expanded) =
                expanded_move_structure(realtime, structure, *offset, turn_structures, mode)
            else {
                return false;
            };
            if super::structures::hard_pusher_head_blocks_move_public(
                &expanded,
                *offset,
                hard_heads,
            ) {
                return false;
            }
            for pos in &expanded {
                if let Some(block) = realtime.blocks.get(pos) {
                    animations.insert(
                        *pos + offset,
                        BlockAnimation {
                            from_pos: *pos,
                            to_pos: *pos + offset,
                            from_facing: block.facing,
                            to_facing: block.facing,
                            kind: BlockAnimationKind::Move,
                            duration: None,
                            progress: None,
                        },
                    );
                }
            }
            if let Some(actor) = actor {
                let (from_extension, to_extension) = match actor.animation {
                    PusherAnimationKind::Extend => (0.0, 1.0),
                    PusherAnimationKind::Retract => (1.0, 0.0),
                };
                pusher_animations.insert(
                    actor.pos,
                    PusherAnimation {
                        duration: 0.0,
                        from_extension,
                        to_extension,
                    },
                );
                let extended = matches!(actor.animation, PusherAnimationKind::Extend);
                pusher_state.set_extended(actor.pos, realtime, extended);
            }
            let before = expanded.clone();
            moved.extend(expanded.iter().copied());
            move_structure(realtime, &expanded, *offset);
            turn_structures.move_positions(&expanded, *offset);
            let target: HashSet<IVec3> = expanded.iter().map(|pos| *pos + offset).collect();
            movement_influence.record_successful_translate(&before, &target);
            moved.extend(target.iter().copied());
            true
        }
        StructureMove::Rotate {
            structure,
            pivot,
            clockwise,
            source,
        } => {
            if structure.iter().any(|pos| moved.contains(pos)) {
                return false;
            }
            if !can_rotate_structure(realtime, structure, *pivot, *clockwise) {
                return false;
            }
            for pos in structure {
                if let Some(block) = realtime.blocks.get(pos) {
                    let target = rotate_pos_y(*pos, *pivot, *clockwise);
                    animations.insert(
                        target,
                        BlockAnimation {
                            from_pos: *pos,
                            to_pos: target,
                            from_facing: block.facing,
                            to_facing: rotate_facing_internal(block.facing, *clockwise),
                            kind: BlockAnimationKind::Rotate {
                                pivot: *pivot,
                                clockwise: *clockwise,
                            },
                            duration: None,
                            progress: None,
                        },
                    );
                }
            }
            let before = structure.clone();
            moved.extend(structure.iter().copied());
            rotate_structure(realtime, structure, *pivot, *clockwise);
            let targets: HashSet<IVec3> = structure
                .iter()
                .map(|pos| rotate_pos_y(*pos, *pivot, *clockwise))
                .collect();
            turn_structures.replace_structure_positions(structure, targets.clone());
            if let Some(source) = source {
                movement_influence.record_successful_rotate(&before, &targets, *source);
            }
            moved.extend(targets);
            true
        }
    }
}

fn blocks_gravity_pusher_head(
    world: &WorldBlocks,
    structure: &HashSet<IVec3>,
    hard_heads: &HashSet<IVec3>,
    offset: IVec3,
) -> bool {
    if offset != IVec3::NEG_Y {
        return false;
    }
    super::structures::hard_pusher_head_blocked_below_public(world, structure, hard_heads)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind};
    use crate::game::world::direction::Facing;

    fn stone(pos: IVec3) -> BlockData {
        BlockData {
            kind: BlockKind::Stone,
            facing: Facing::North,
        }
    }

    fn platform() -> BlockData {
        BlockData {
            kind: BlockKind::Platform,
            facing: Facing::North,
        }
    }

    fn pusher(facing: Facing) -> BlockData {
        BlockData {
            kind: BlockKind::Pusher,
            facing,
        }
    }

    fn conveyor(facing: Facing) -> BlockData {
        BlockData {
            kind: BlockKind::Conveyor,
            facing,
        }
    }

    fn material() -> BlockData {
        BlockData {
            kind: BlockKind::Material,
            facing: Facing::North,
        }
    }

    fn rebuild_structures(world: &WorldBlocks) -> StructureState {
        let mut state = StructureState::default();
        state.rebuild_for_simulation(world);
        state
    }

    #[test]
    fn opposing_pushers_only_one_occupies_shared_head_cell() {
        let left = IVec3::new(0, 1, 0);
        let middle = IVec3::new(1, 1, 0);
        let right = IVec3::new(2, 1, 0);
        let mut world = WorldBlocks::default();
        for pos in [left, middle, right] {
            world.insert(pos + IVec3::NEG_Y, stone(pos + IVec3::NEG_Y));
        }
        world.insert(left, pusher(Facing::East));
        world.insert(middle, platform());
        world.insert(right, pusher(Facing::West));

        let solution = world.clone();
        let solution_structures = rebuild_structures(&solution);
        let mut turn_structures = solution_structures.clone();
        let mut pusher_state = PusherState::default();
        let mut movement_influence = MovementInfluenceCache::default();
        let powered = HashSet::from([left, right]);

        let plan = collect_movement_plan(
            &world,
            &solution,
            &mut turn_structures,
            &solution_structures,
            &powered,
            &pusher_state,
            &mut movement_influence,
        );
        let mut realtime = world.clone();
        execute_movement_plan(
            &plan,
            &mut realtime,
            &mut turn_structures,
            &mut pusher_state,
            &mut movement_influence,
        );

        let heads = pusher_state.hard_head_occupancy(&realtime);
        assert_eq!(heads, HashSet::from([middle]));
    }

    #[test]
    fn conveyor_stops_when_forward_cell_is_pusher_head() {
        let conveyor_pos = IVec3::new(-1, 1, 0);
        let material_pos = IVec3::new(0, 1, 0);
        let blocked = IVec3::new(1, 1, 0);
        let pusher_pos = IVec3::new(2, 1, 0);
        let mut world = WorldBlocks::default();
        for pos in [conveyor_pos, material_pos, blocked, pusher_pos] {
            world.insert(pos + IVec3::NEG_Y, stone(pos + IVec3::NEG_Y));
        }
        world.insert(conveyor_pos, conveyor(Facing::East));
        world.insert(material_pos, material());
        world.insert(pusher_pos, pusher(Facing::West));

        let solution = world.clone();
        let solution_structures = rebuild_structures(&solution);
        let mut turn_structures = solution_structures.clone();
        let mut pusher_state = PusherState::default();
        pusher_state.set_extended(pusher_pos, &world, true);
        let mut movement_influence = MovementInfluenceCache::default();

        let plan = collect_movement_plan(
            &world,
            &solution,
            &mut turn_structures,
            &solution_structures,
            &HashSet::new(),
            &pusher_state,
            &mut movement_influence,
        );
        let mut realtime = world.clone();
        execute_movement_plan(
            &plan,
            &mut realtime,
            &mut turn_structures,
            &mut pusher_state,
            &mut movement_influence,
        );

        assert!(realtime.is_material_at(material_pos));
        assert!(!realtime.is_material_at(blocked));
    }
}
