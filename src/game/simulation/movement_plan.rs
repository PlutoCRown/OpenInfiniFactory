use bevy::prelude::*;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::game::world::animation::{BlockAnimation, BlockAnimationKind, PusherAnimation};
use crate::game::world::factory_registry::FactoryBlockRegistry;
use crate::game::world::grid::WorldBlocks;

use super::gravity::mark_gravity_phase;
use super::movement::{
    collect_conveyor_candidate, collect_lift_candidate, collect_powered_translate_candidate,
    collect_pusher_candidate, collect_rotate_candidate, pusher_head_position, PusherState,
};
use super::runtime::SignalNetworkCache;
use super::structure_state::{material_structure, query_factory_structure, StructureState};
use super::structures::{
    can_rotate_structure, expanded_move_structure, move_structure, rotate_facing_internal,
    rotate_pos_y, rotate_structure, MovementCandidate, MovementInfluenceCache, MovementMark,
    PusherAnimationKind, StructureMove, StructureMovePhaseKind,
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
    factory_registry: &FactoryBlockRegistry,
    powered_devices: &HashSet<IVec3>,
    pusher_state: &PusherState,
    movement_influence: &mut MovementInfluenceCache,
) -> MovementPlan {
    let mut rotate = Vec::new();
    let mut push = Vec::new();
    let mut lift = Vec::new();
    let mut bare_pusher_animations = HashMap::new();

    {
        let ctx = super::movement::MovementMarkContext {
            turn,
            solution,
            turn_structures,
            solution_structures,
            factory_registry,
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
                        crate::game::blocks::BlockKind::Pusher
                            | crate::game::blocks::BlockKind::Blocker
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
                crate::game::blocks::MovementRule::Translate { .. } => {}
            }
        }
    }

    let mut conveyor = Vec::new();
    {
        let ctx = super::movement::MovementMarkContext {
            turn,
            solution,
            turn_structures,
            solution_structures,
            factory_registry,
        };
        for (pos, _kind, rule) in super::movement::sorted_factory_movers(turn) {
            let crate::game::blocks::MovementRule::Translate { source, offset } = rule else {
                continue;
            };
            if let Some(candidate) = collect_conveyor_candidate(&ctx, pos, source, offset) {
                conveyor.push(candidate);
            }
        }
    }

    let actuating = pusher_state.actuating_devices(turn, powered_devices);
    let hard_heads = pusher_state.hard_head_occupancy(turn);
    let gravity = mark_gravity_phase(turn, turn_structures, &actuating, &hard_heads)
        .into_iter()
        .map(|movement| MovementCandidate {
            primary: movement,
            fallbacks: Vec::new(),
        })
        .collect::<Vec<_>>();

    movement_influence.begin_turn_from_candidates(&rotate, &conveyor);
    rotate.retain(|candidate| !movement_influence.ignores_rotator_movement(&candidate.primary));
    conveyor.sort_by(|a, b| movement_influence.compare_conveyor_candidates(a, b));
    movement_influence.finish_turn_from_candidates(&conveyor);

    let (conveyor_transport, conveyor_self) = split_conveyor_candidates(conveyor);

    let mut plan = MovementPlan::default();
    plan.phases.push(MovementPhasePlan {
        phase: StructureMovePhaseKind::Fixed,
        candidates: Vec::new(),
    });
    plan.phases
        .push(sorted_phase(StructureMovePhaseKind::Rotate, rotate));
    plan.phases
        .push(sorted_phase(StructureMovePhaseKind::Push, push));
    plan.phases
        .push(sorted_phase(StructureMovePhaseKind::Lift, lift));
    plan.phases.push(sorted_phase(
        StructureMovePhaseKind::Conveyor,
        conveyor_transport,
    ));
    plan.phases
        .push(sorted_phase(StructureMovePhaseKind::Gravity, gravity));
    plan.phases.push(sorted_phase(
        StructureMovePhaseKind::Conveyor,
        conveyor_self,
    ));
    plan.bare_pusher_animations = bare_pusher_animations;
    plan
}

fn split_conveyor_candidates(
    conveyor: Vec<MovementCandidate>,
) -> (Vec<MovementCandidate>, Vec<MovementCandidate>) {
    let mut transport = Vec::new();
    let mut self_moves = Vec::new();
    for candidate in conveyor {
        if candidate.fallbacks.is_empty() {
            self_moves.push(candidate);
        } else {
            transport.push(MovementCandidate {
                primary: candidate.primary,
                fallbacks: Vec::new(),
            });
            self_moves.push(MovementCandidate {
                primary: candidate.fallbacks[0].clone(),
                fallbacks: Vec::new(),
            });
        }
    }
    (transport, self_moves)
}

fn sorted_phase(
    phase: StructureMovePhaseKind,
    mut candidates: Vec<MovementCandidate>,
) -> MovementPhasePlan {
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
    factory_registry: &mut FactoryBlockRegistry,
    pusher_state: &mut PusherState,
    movement_influence: &mut MovementInfluenceCache,
) -> MovementExecutionOutput {
    let mut moved = HashSet::new();
    let mut animations = HashMap::new();
    let mut pusher_animations = HashMap::new();

    for phase in &plan.phases {
        let mut occupied_heads = if phase.phase == StructureMovePhaseKind::Push {
            pusher_state.hard_head_occupancy(realtime)
        } else {
            HashSet::new()
        };
        let hard_heads = occupied_heads.clone();

        for candidate in &phase.candidates {
            for movement in std::iter::once(&candidate.primary).chain(candidate.fallbacks.iter()) {
                if try_execute_move(
                    realtime,
                    turn_structures,
                    factory_registry,
                    pusher_state,
                    movement_influence,
                    &hard_heads,
                    &mut occupied_heads,
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
            let mut bare_pushers: Vec<_> = plan.bare_pusher_animations.iter().collect();
            bare_pushers.sort_by_key(|(pos, _)| (pos.x, pos.y, pos.z));
            for (&pos, animation) in bare_pushers {
                if phase
                    .candidates
                    .iter()
                    .any(|candidate| candidate.primary.pusher_actor() == Some(pos))
                {
                    continue;
                }
                if pusher_animations.contains_key(&pos) {
                    continue;
                }
                let Some(head) = pusher_head_position(realtime, pos) else {
                    continue;
                };
                if animation.to_extension <= animation.from_extension {
                    pusher_animations.insert(pos, *animation);
                    apply_pusher_extension_from_animation(pusher_state, realtime, pos, animation);
                    continue;
                }
                if occupied_heads.contains(&head) {
                    continue;
                }
                occupied_heads.insert(head);
                pusher_animations.insert(pos, *animation);
                apply_pusher_extension_from_animation(pusher_state, realtime, pos, animation);
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
    factory_registry: &mut FactoryBlockRegistry,
    pusher_state: &mut PusherState,
    movement_influence: &mut MovementInfluenceCache,
    hard_heads: &HashSet<IVec3>,
    occupied_heads: &mut HashSet<IVec3>,
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
            let head_blocks = if let Some(actor) = actor {
                if matches!(actor.animation, PusherAnimationKind::Retract) {
                    let Some(head) = pusher_head_position(realtime, actor.pos) else {
                        return false;
                    };
                    super::structures::hard_pusher_head_blocks_move_public_excluding(
                        &expanded, *offset, hard_heads, head,
                    )
                } else {
                    super::structures::hard_pusher_head_blocks_move_public(
                        &expanded, *offset, hard_heads,
                    )
                }
            } else {
                super::structures::hard_pusher_head_blocks_move_public(
                    &expanded, *offset, hard_heads,
                )
            };
            if head_blocks {
                return false;
            }
            if let Some(actor) = actor {
                let extending = matches!(actor.animation, PusherAnimationKind::Extend);
                if extending {
                    let Some(head) = pusher_head_position(realtime, actor.pos) else {
                        return false;
                    };
                    if occupied_heads.contains(&head) {
                        return false;
                    }
                }
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
                let extending = matches!(actor.animation, PusherAnimationKind::Extend);
                if extending {
                    let Some(head) = pusher_head_position(realtime, actor.pos) else {
                        return false;
                    };
                    if occupied_heads.contains(&head) {
                        return false;
                    }
                    occupied_heads.insert(head);
                } else if let Some(head) = pusher_head_position(realtime, actor.pos) {
                    occupied_heads.remove(&head);
                }
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
                pusher_state.set_extended(actor.pos, realtime, extending);
            }
            let before = expanded.clone();
            moved.extend(expanded.iter().copied());
            move_structure(realtime, &expanded, *offset);
            turn_structures.move_positions(&expanded, *offset);
            factory_registry.translate_turn(&before, *offset);
            pusher_state.translate_device_entries(&before, *offset);
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
            factory_registry.rotate_turn(structure, *pivot, *clockwise);
            pusher_state.rotate_device_entries(structure, *pivot, *clockwise);
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

pub fn preview_movement_plan(
    turn: &WorldBlocks,
    solution: &WorldBlocks,
    turn_structures: &mut StructureState,
    solution_structures: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    powered_devices: &HashSet<IVec3>,
    pusher_state: &PusherState,
    movement_influence: &mut MovementInfluenceCache,
) -> MovementPlan {
    collect_movement_plan(
        turn,
        solution,
        turn_structures,
        solution_structures,
        factory_registry,
        powered_devices,
        pusher_state,
        movement_influence,
    )
}

pub fn target_structure_movement_lines(
    target_pos: IVec3,
    turn: &WorldBlocks,
    turn_structures: &StructureState,
    solution: &WorldBlocks,
    solution_structures: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    signal_cache: &mut SignalNetworkCache,
    pusher_state: &PusherState,
    movement_influence: &MovementInfluenceCache,
) -> Option<Vec<String>> {
    let structure = aimed_structure_at(target_pos, turn, turn_structures)?;
    signal_cache.ensure_fresh(turn);
    let powered = signal_cache.powered_device_positions(turn);
    let mut preview_structures = turn_structures.clone();
    let mut preview_influence = movement_influence.clone();
    let plan = preview_movement_plan(
        turn,
        solution,
        &mut preview_structures,
        solution_structures,
        factory_registry,
        &powered,
        pusher_state,
        &mut preview_influence,
    );
    Some(collect_matching_movement_lines(&plan, &structure))
}

fn aimed_structure_at(
    pos: IVec3,
    world: &WorldBlocks,
    structure_state: &StructureState,
) -> Option<HashSet<IVec3>> {
    let block = world.blocks.get(&pos)?;
    if block.kind.is_scene() {
        return None;
    }
    if block.kind.is_material() {
        return Some(
            structure_state
                .pushable_structure_at(pos, IVec3::ZERO)
                .unwrap_or_else(|| material_structure(world, pos)),
        );
    }
    if block.kind.is_factory() {
        return structure_state
            .movable_structure_at(pos)
            .or_else(|| query_factory_structure(world, pos));
    }
    None
}

fn collect_matching_movement_lines(plan: &MovementPlan, structure: &HashSet<IVec3>) -> Vec<String> {
    let mut lines = Vec::new();
    for phase in &plan.phases {
        for candidate in &phase.candidates {
            if movement_matches_structure(&candidate.primary, structure) {
                lines.push(format_movement_line(phase.phase, &candidate.primary));
            }
            for fallback in &candidate.fallbacks {
                if movement_matches_structure(fallback, structure) {
                    lines.push(format_movement_line(phase.phase, fallback));
                }
            }
        }
    }
    lines
}

fn movement_matches_structure(movement: &StructureMove, structure: &HashSet<IVec3>) -> bool {
    match movement {
        StructureMove::Translate {
            structure: positions,
            ..
        }
        | StructureMove::Rotate {
            structure: positions,
            ..
        } => positions == structure,
    }
}

fn format_movement_line(phase: StructureMovePhaseKind, movement: &StructureMove) -> String {
    let phase = phase_kind_label(phase);
    match movement {
        StructureMove::Translate {
            offset,
            mark,
            source,
            ..
        } => {
            let kind = mark_label(*mark);
            format!(
                "[{phase}] {kind} <- {source}  d({dx}, {dy}, {dz})",
                source = format_source(*source),
                dx = offset.x,
                dy = offset.y,
                dz = offset.z,
            )
        }
        StructureMove::Rotate {
            pivot,
            clockwise,
            source,
            ..
        } => {
            let direction = if *clockwise { "CW" } else { "CCW" };
            format!(
                "[{phase}] Rotate @ ({px}, {py}, {pz}) {direction} <- {source}",
                px = pivot.x,
                py = pivot.y,
                pz = pivot.z,
                source = format_source(*source),
            )
        }
    }
}

fn format_source(source: Option<IVec3>) -> String {
    source
        .map(|pos| format!("({}, {}, {})", pos.x, pos.y, pos.z))
        .unwrap_or_else(|| "—".to_string())
}

pub(crate) fn preview_candidate_executable(
    movement: &StructureMove,
    turn: &WorldBlocks,
    turn_structures: &StructureState,
    hard_heads: &HashSet<IVec3>,
) -> (bool, &'static str) {
    match movement {
        StructureMove::Translate {
            structure,
            offset,
            mark,
            source,
            ..
        } => {
            if blocks_gravity_pusher_head(turn, structure, hard_heads, *offset) {
                return (false, "blocked_by_pusher_head");
            }
            let mode = super::structures::movement_expansion_mode_public(*mark, *source);
            if expanded_move_structure(turn, structure, *offset, turn_structures, mode).is_some() {
                (true, "ok")
            } else {
                (false, "cannot_translate")
            }
        }
        StructureMove::Rotate {
            structure,
            pivot,
            clockwise,
            ..
        } => {
            if can_rotate_structure(turn, structure, *pivot, *clockwise) {
                (true, "ok")
            } else {
                (false, "cannot_rotate")
            }
        }
    }
}

pub fn phase_kind_label(phase: StructureMovePhaseKind) -> &'static str {
    match phase {
        StructureMovePhaseKind::Fixed => "Fixed",
        StructureMovePhaseKind::Rotate => "Rotate",
        StructureMovePhaseKind::Push => "Push",
        StructureMovePhaseKind::Lift => "Lift",
        StructureMovePhaseKind::Gravity => "Gravity",
        StructureMovePhaseKind::Conveyor => "Conveyor",
    }
}

pub(crate) fn mark_label(mark: MovementMark) -> &'static str {
    match mark {
        MovementMark::Conveyor => "Conveyor",
        MovementMark::Push => "Push",
        MovementMark::Vertical => "Gravity",
    }
}

fn offset_json(offset: IVec3) -> Value {
    json!({ "x": offset.x, "y": offset.y, "z": offset.z })
}

fn positions_json(positions: &HashSet<IVec3>) -> Value {
    let mut list: Vec<_> = positions
        .iter()
        .map(|pos| json!({ "x": pos.x, "y": pos.y, "z": pos.z }))
        .collect();
    list.sort_by_key(|pos| {
        (
            pos["x"].as_i64().unwrap_or(0),
            pos["y"].as_i64().unwrap_or(0),
            pos["z"].as_i64().unwrap_or(0),
        )
    });
    Value::Array(list)
}

fn movement_preview_json(
    movement: &StructureMove,
    turn: &WorldBlocks,
    turn_structures: &StructureState,
    hard_heads: &HashSet<IVec3>,
) -> Value {
    let (executable, reason) =
        preview_candidate_executable(movement, turn, turn_structures, hard_heads);
    match movement {
        StructureMove::Translate {
            structure,
            offset,
            mark,
            source,
            actor,
        } => json!({
            "kind": "Translate",
            "mark": mark_label(*mark),
            "offset": offset_json(*offset),
            "source": source.map(|pos| json!({ "x": pos.x, "y": pos.y, "z": pos.z })),
            "pusher_actor": actor.as_ref().map(|actor| json!({ "x": actor.pos.x, "y": actor.pos.y, "z": actor.pos.z })),
            "member_count": structure.len(),
            "members": positions_json(structure),
            "executable": executable,
            "reason": reason,
        }),
        StructureMove::Rotate {
            structure,
            pivot,
            clockwise,
            source,
        } => json!({
            "kind": "Rotate",
            "mark": "Rotate",
            "pivot": json!({ "x": pivot.x, "y": pivot.y, "z": pivot.z }),
            "clockwise": clockwise,
            "source": source.map(|pos| json!({ "x": pos.x, "y": pos.y, "z": pos.z })),
            "member_count": structure.len(),
            "members": positions_json(structure),
            "executable": executable,
            "reason": reason,
        }),
    }
}

pub fn movement_plan_debug_json(
    turn: &WorldBlocks,
    turn_structures: &StructureState,
    solution: &WorldBlocks,
    solution_structures: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    signal_cache: &mut super::runtime::SignalNetworkCache,
    pusher_state: &PusherState,
    movement_influence: &mut MovementInfluenceCache,
    turn_number: u64,
) -> Value {
    let mut turn_structures = turn_structures.clone();
    signal_cache.ensure_fresh(turn);
    let powered = signal_cache.powered_device_positions(turn);
    let hard_heads = pusher_state.extended_head_positions(turn);
    let plan = preview_movement_plan(
        turn,
        solution,
        &mut turn_structures,
        solution_structures,
        factory_registry,
        &powered,
        pusher_state,
        movement_influence,
    );
    let phases: Vec<_> = plan
        .phases
        .iter()
        .map(|phase| {
            let candidates: Vec<_> = phase
                .candidates
                .iter()
                .map(|candidate| {
                    json!({
                        "primary": movement_preview_json(
                            &candidate.primary,
                            turn,
                            &turn_structures,
                            &hard_heads,
                        ),
                        "fallbacks": candidate.fallbacks.iter().map(|movement| {
                            movement_preview_json(movement, turn, &turn_structures, &hard_heads)
                        }).collect::<Vec<_>>(),
                    })
                })
                .collect();
            json!({
                "phase": phase_kind_label(phase.phase),
                "candidate_count": candidates.len(),
                "candidates": candidates,
            })
        })
        .collect();
    let bare_pushers: Vec<_> = plan
        .bare_pusher_animations
        .iter()
        .map(|(pos, animation)| {
            json!({
                "pos": { "x": pos.x, "y": pos.y, "z": pos.z },
                "from_extension": animation.from_extension,
                "to_extension": animation.to_extension,
            })
        })
        .collect();
    json!({
        "turn": turn_number,
        "powered_device_count": powered.len(),
        "powered_devices": powered.iter().map(|pos| json!({ "x": pos.x, "y": pos.y, "z": pos.z })).collect::<Vec<_>>(),
        "extended_pusher_heads": hard_heads.iter().map(|pos| json!({ "x": pos.x, "y": pos.y, "z": pos.z })).collect::<Vec<_>>(),
        "bare_pusher_animations": bare_pushers,
        "phases": phases,
    })
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

    fn reverse_conveyor(facing: Facing) -> BlockData {
        BlockData {
            kind: BlockKind::ReverseConveyor,
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

    fn frozen_registry(world: &WorldBlocks) -> FactoryBlockRegistry {
        let mut registry = FactoryBlockRegistry::rebuild_from_world(world);
        registry.freeze_solution();
        registry
    }

    #[test]
    fn four_converging_bare_pushers_only_one_occupies_shared_head_cell() {
        let center = IVec3::new(1, 1, 0);
        let north = IVec3::new(1, 1, -1);
        let south = IVec3::new(1, 1, 1);
        let east = IVec3::new(2, 1, 0);
        let west = IVec3::new(0, 1, 0);
        let mut world = WorldBlocks::default();
        for pos in [north, south, east, west, center] {
            world.insert(pos + IVec3::NEG_Y, stone(pos + IVec3::NEG_Y));
        }
        world.insert(north, pusher(Facing::South));
        world.insert(south, pusher(Facing::North));
        world.insert(east, pusher(Facing::West));
        world.insert(west, pusher(Facing::East));

        let solution = world.clone();
        let solution_structures = rebuild_structures(&solution);
        let mut turn_structures = solution_structures.clone();
        let mut pusher_state = PusherState::default();
        let mut movement_influence = MovementInfluenceCache::default();
        let powered = HashSet::from([north, south, east, west]);
        let mut factory_registry = frozen_registry(&world);

        let plan = collect_movement_plan(
            &world,
            &solution,
            &mut turn_structures,
            &solution_structures,
            &factory_registry,
            &powered,
            &pusher_state,
            &mut movement_influence,
        );
        assert_eq!(plan.bare_pusher_animations.len(), 4);
        assert!(plan.phases[2].candidates.is_empty());

        let mut realtime = world.clone();
        let output = execute_movement_plan(
            &plan,
            &mut realtime,
            &mut turn_structures,
            &mut factory_registry,
            &mut pusher_state,
            &mut movement_influence,
        );

        let heads = pusher_state.hard_head_occupancy(&realtime);
        assert_eq!(heads, HashSet::from([center]));
        assert_eq!(
            output
                .pusher_animations
                .values()
                .filter(|animation| animation.to_extension > animation.from_extension)
                .count(),
            1
        );
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
        let mut factory_registry = frozen_registry(&world);

        let plan = collect_movement_plan(
            &world,
            &solution,
            &mut turn_structures,
            &solution_structures,
            &factory_registry,
            &powered,
            &pusher_state,
            &mut movement_influence,
        );
        let mut realtime = world.clone();
        execute_movement_plan(
            &plan,
            &mut realtime,
            &mut turn_structures,
            &mut factory_registry,
            &mut pusher_state,
            &mut movement_influence,
        );

        let heads = pusher_state.hard_head_occupancy(&realtime);
        assert!(heads.is_empty());
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
        let mut factory_registry = frozen_registry(&world);

        let plan = collect_movement_plan(
            &world,
            &solution,
            &mut turn_structures,
            &solution_structures,
            &factory_registry,
            &HashSet::new(),
            &pusher_state,
            &mut movement_influence,
        );
        let mut realtime = world.clone();
        execute_movement_plan(
            &plan,
            &mut realtime,
            &mut turn_structures,
            &mut factory_registry,
            &mut pusher_state,
            &mut movement_influence,
        );

        assert!(realtime.is_material_at(material_pos));
        assert!(!realtime.is_material_at(blocked));
    }

    #[test]
    fn reverse_conveyor_does_not_self_push_when_workface_is_empty() {
        let conveyor_pos = IVec3::new(0, 2, 0);
        let mut world = WorldBlocks::default();
        world.insert(conveyor_pos, reverse_conveyor(Facing::North));

        let solution = world.clone();
        let solution_structures = rebuild_structures(&solution);
        let mut turn_structures = solution_structures.clone();
        let pusher_state = PusherState::default();
        let mut movement_influence = MovementInfluenceCache::default();
        let factory_registry = frozen_registry(&world);

        let plan = collect_movement_plan(
            &world,
            &solution,
            &mut turn_structures,
            &solution_structures,
            &factory_registry,
            &HashSet::new(),
            &pusher_state,
            &mut movement_influence,
        );

        let conveyor_moves: Vec<_> = plan
            .phases
            .iter()
            .filter(|phase| phase.phase == StructureMovePhaseKind::Conveyor)
            .flat_map(|phase| phase.candidates.iter())
            .flat_map(|candidate| {
                std::iter::once(&candidate.primary).chain(candidate.fallbacks.iter())
            })
            .filter(|movement| movement.structure().contains(&conveyor_pos))
            .collect();
        assert!(conveyor_moves.is_empty());
    }

    #[test]
    fn north_reverse_conveyor_on_stone_pushes_itself() {
        let start = IVec3::new(0, 2, 0);
        let after_gravity = IVec3::new(0, 1, 0);
        let after_self_push = IVec3::new(0, 1, -1);
        let after_second_gravity = IVec3::new(0, 0, -1);
        let mut world = WorldBlocks::default();
        world.insert(IVec3::ZERO, stone(IVec3::ZERO));
        world.insert(start, reverse_conveyor(Facing::North));

        let solution = world.clone();
        let solution_structures = rebuild_structures(&solution);
        let mut turn_structures = solution_structures.clone();
        let mut pusher_state = PusherState::default();
        let mut movement_influence = MovementInfluenceCache::default();
        let mut factory_registry = frozen_registry(&world);

        let mut realtime = world.clone();
        for (expected, unexpected) in [
            (after_gravity, start),
            (after_self_push, after_gravity),
            (after_second_gravity, after_self_push),
        ] {
            let plan = collect_movement_plan(
                &realtime,
                &solution,
                &mut turn_structures,
                &solution_structures,
                &factory_registry,
                &HashSet::new(),
                &pusher_state,
                &mut movement_influence,
            );
            execute_movement_plan(
                &plan,
                &mut realtime,
                &mut turn_structures,
                &mut factory_registry,
                &mut pusher_state,
                &mut movement_influence,
            );

            assert!(realtime
                .blocks
                .get(&expected)
                .is_some_and(|block| block.kind == BlockKind::ReverseConveyor));
            assert!(!realtime.blocks.contains_key(&unexpected));
        }
    }

    #[test]
    fn south_reverse_conveyor_moves_platform_from_below_back() {
        let conveyor_pos = IVec3::new(0, 4, 1);
        let platform_pos = IVec3::new(0, 3, 1);
        let target = IVec3::new(0, 3, 0);
        let mut world = WorldBlocks::default();
        world.insert(IVec3::ZERO, stone(IVec3::ZERO));
        world.insert(platform_pos, platform());
        world.insert(conveyor_pos, reverse_conveyor(Facing::South));

        let solution = world.clone();
        let solution_structures = rebuild_structures(&solution);
        let mut turn_structures = solution_structures.clone();
        let mut pusher_state = PusherState::default();
        let mut movement_influence = MovementInfluenceCache::default();
        let mut factory_registry = frozen_registry(&world);

        let plan = collect_movement_plan(
            &world,
            &solution,
            &mut turn_structures,
            &solution_structures,
            &factory_registry,
            &HashSet::new(),
            &pusher_state,
            &mut movement_influence,
        );
        let mut realtime = world.clone();
        execute_movement_plan(
            &plan,
            &mut realtime,
            &mut turn_structures,
            &mut factory_registry,
            &mut pusher_state,
            &mut movement_influence,
        );

        assert!(realtime
            .blocks
            .get(&target)
            .is_some_and(|block| block.kind == BlockKind::Platform));
        assert!(!realtime
            .blocks
            .get(&platform_pos)
            .is_some_and(|block| block.kind == BlockKind::Platform));
    }

    fn mye2e_pusher_2_world() -> WorldBlocks {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::new(-12, 0, 12), stone(IVec3::new(-12, 0, 12)));
        for (pos, kind, facing) in [
            (IVec3::new(-12, 0, 11), BlockKind::Platform, Facing::West),
            (IVec3::new(-10, 1, 10), BlockKind::Platform, Facing::West),
            (IVec3::new(-10, 0, 10), BlockKind::Conveyor, Facing::West),
            (IVec3::new(-12, 4, 12), BlockKind::Pusher, Facing::North),
            (IVec3::new(-11, 0, 10), BlockKind::Conveyor, Facing::West),
            (IVec3::new(-13, 0, 12), BlockKind::Platform, Facing::West),
            (IVec3::new(-12, 0, 10), BlockKind::Platform, Facing::West),
            (
                IVec3::new(-12, 4, 11),
                BlockKind::ReverseConveyor,
                Facing::West,
            ),
            (IVec3::new(-11, 0, 12), BlockKind::Platform, Facing::West),
            (IVec3::new(-11, 1, 12), BlockKind::Platform, Facing::West),
            (IVec3::new(-12, 1, 11), BlockKind::Detector, Facing::North),
            (IVec3::new(-12, 3, 12), BlockKind::Wire, Facing::West),
            (IVec3::new(-13, 1, 12), BlockKind::Platform, Facing::West),
        ] {
            world.insert(pos, BlockData { kind, facing });
        }
        world
    }

    #[test]
    fn mye2e_pusher_target_structure_before_and_after_gravity() {
        use super::super::core::simulate_turn;

        let start_world = mye2e_pusher_2_world();
        let structures = rebuild_structures(&start_world);
        let mut worlds = super::super::worlds::SimulationWorlds::at_simulation_start(
            start_world.clone(),
            structures,
        );
        let pusher_pos = IVec3::new(-12, 4, 12);
        let target_pos = IVec3::new(-12, 4, 11);
        let offset = IVec3::new(0, 0, -1);
        let before = worlds.solution_structures.pusher_target_structure(
            &worlds.solution,
            &worlds.factory_registry,
            pusher_pos,
            target_pos,
            offset,
        );
        assert_eq!(before, Some(HashSet::from([target_pos])));

        let mut pending = super::super::runtime::PendingGeneratedMaterials::default();
        let mut signal_cache = super::super::runtime::SignalNetworkCache::default();
        let mut pusher_state = PusherState::rebuild_from_world(&worlds.turn);
        let mut movement_influence = MovementInfluenceCache::default();
        for turn in 0..2 {
            simulate_turn(
                &mut worlds,
                &mut pending,
                &mut signal_cache,
                turn,
                0.0,
                &mut pusher_state,
                &mut movement_influence,
                None,
                None,
            );
        }

        let actual_pusher = worlds
            .turn
            .blocks
            .iter()
            .find_map(|(pos, block)| (block.kind == BlockKind::Pusher).then_some(*pos))
            .unwrap();
        let fallen_target = IVec3::new(-12, 2, 11);
        let actual_rcx = worlds
            .turn
            .blocks
            .iter()
            .find_map(|(pos, block)| (block.kind == BlockKind::ReverseConveyor).then_some(*pos))
            .unwrap();
        assert_eq!(
            actual_pusher,
            IVec3::new(-12, 2, 12),
            "unexpected pusher position"
        );
        assert_eq!(
            actual_rcx, fallen_target,
            "unexpected reverse conveyor position"
        );
        assert!(worlds.factory_registry.has_turn_factory(actual_pusher));
        assert!(worlds.factory_registry.has_turn_factory(actual_rcx));
        let after = worlds.solution_structures.pusher_target_structure(
            &worlds.solution,
            &worlds.factory_registry,
            actual_pusher,
            actual_rcx,
            offset,
        );
        assert_eq!(after, Some(HashSet::from([fallen_target])));
    }

    #[test]
    fn mye2e_pusher_2_platform_oscillates_between_minus_11_and_minus_12() {
        use super::super::core::simulate_turn;
        use super::super::runtime::{PendingGeneratedMaterials, SignalNetworkCache};

        let start_world = mye2e_pusher_2_world();
        let structures = rebuild_structures(&start_world);
        let mut worlds = super::super::worlds::SimulationWorlds::at_simulation_start(
            start_world.clone(),
            structures,
        );
        let mut pending = PendingGeneratedMaterials::default();
        let mut signal_cache = SignalNetworkCache::default();
        let mut pusher_state = PusherState::rebuild_from_world(&worlds.turn);
        let mut movement_influence = MovementInfluenceCache::default();

        let tracked = IVec3::new(-10, 1, 10);
        let mut positions = Vec::new();
        for turn in 0..10 {
            simulate_turn(
                &mut worlds,
                &mut pending,
                &mut signal_cache,
                turn,
                0.0,
                &mut pusher_state,
                &mut movement_influence,
                None,
                None,
            );
            let pos = worlds
                .turn
                .blocks
                .iter()
                .find_map(|(pos, block)| {
                    (block.kind == BlockKind::Platform
                        && pos.x >= -12
                        && pos.x <= -10
                        && pos.y == 1
                        && pos.z == 10)
                        .then_some(*pos)
                })
                .unwrap_or(tracked);
            positions.push(pos);
        }

        assert!(
            positions.contains(&IVec3::new(-11, 1, 10)),
            "platform never reached (-11,1,10); path: {:?}",
            positions
        );
        assert!(
            positions.contains(&IVec3::new(-12, 1, 10)),
            "platform never reached (-12,1,10); path: {:?}",
            positions
        );
        let oscillates = positions.windows(2).any(|window| {
            (window[0] == IVec3::new(-12, 1, 10) && window[1] == IVec3::new(-11, 1, 10))
                || (window[0] == IVec3::new(-11, 1, 10) && window[1] == IVec3::new(-12, 1, 10))
        });
        assert!(
            oscillates,
            "platform did not oscillate between (-11,1,10) and (-12,1,10); path: {:?}",
            positions
        );
    }

    fn mye2e_pusher_back_2_world() -> WorldBlocks {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::new(2, 0, 1), stone(IVec3::new(2, 0, 1)));
        world.insert(IVec3::new(2, 1, 0), platform());
        world.insert(
            IVec3::new(1, 1, 1),
            BlockData {
                kind: BlockKind::Detector,
                facing: Facing::West,
            },
        );
        world.insert(IVec3::new(2, 1, 1), pusher(Facing::North));
        world.insert(
            IVec3::new(1, 2, 1),
            BlockData {
                kind: BlockKind::Wire,
                facing: Facing::North,
            },
        );
        world.insert(
            IVec3::new(2, 2, 1),
            BlockData {
                kind: BlockKind::Wire,
                facing: Facing::North,
            },
        );
        world.insert(IVec3::new(0, 3, 1), platform());
        world
    }

    #[test]
    fn mye2e_pusher_back_2_retracts_after_single_power_pulse() {
        use super::super::core::simulate_turn;
        use super::super::runtime::{PendingGeneratedMaterials, SignalNetworkCache};

        let start_world = mye2e_pusher_back_2_world();
        let structures = rebuild_structures(&start_world);
        let mut worlds = super::super::worlds::SimulationWorlds::at_simulation_start(
            start_world.clone(),
            structures,
        );
        let mut pending = PendingGeneratedMaterials::default();
        let mut signal_cache = SignalNetworkCache::default();
        let mut pusher_state = PusherState::rebuild_from_world(&worlds.turn);
        let mut movement_influence = MovementInfluenceCache::default();
        let pusher_pos = IVec3::new(2, 1, 1);

        for turn in 0..3 {
            simulate_turn(
                &mut worlds,
                &mut pending,
                &mut signal_cache,
                turn,
                0.0,
                &mut pusher_state,
                &mut movement_influence,
                None,
                None,
            );
        }
        assert!(
            pusher_state.is_device_extended(pusher_pos),
            "pusher should extend after the single powered turn"
        );
        assert!(
            !worlds.turn.blocks.contains_key(&IVec3::new(2, 1, 0)),
            "platform should leave the cell in front of the pusher when pushed"
        );
        assert!(
            worlds.turn.blocks.contains_key(&IVec3::new(2, 1, -1)),
            "platform should be pushed one cell north"
        );

        simulate_turn(
            &mut worlds,
            &mut pending,
            &mut signal_cache,
            3,
            0.0,
            &mut pusher_state,
            &mut movement_influence,
            None,
            None,
        );
        assert!(
            !pusher_state.is_device_extended(pusher_pos),
            "pusher should retract once power is removed"
        );
        assert!(
            worlds
                .turn
                .blocks
                .get(&IVec3::new(2, 1, 0))
                .is_some_and(|block| block.kind == BlockKind::Platform),
            "bound platform should be pulled back in front of the pusher"
        );
        assert!(
            !worlds.turn.blocks.contains_key(&IVec3::new(2, 1, -1)),
            "platform should not remain at the pushed position"
        );
    }
}
