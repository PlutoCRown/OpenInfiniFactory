use bevy::prelude::*;
use serde_json::{Value, json};

use crate::game::blocks::BlockKind;
use crate::game::simulation::stats::SimulationStepStats;
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    WorldEntryMode,
};
use crate::game::systems::perf::{PerfScope, PerfStats};
use crate::game::ui::UiRuntime;
use crate::game::world::direction::Facing;
use crate::game::world::grid::{TargetHit, WorldBlocks};
use crate::shared::save::{SaveKind, SaveState};
use oif_sim::SimulationControl;
use oif_sim::blocks::{
    MaterialBlockId, PaintMaterialId, StampMaterialId, material_catalog, paint_catalog,
    scene_catalog, stamp_catalog,
};
use oif_sim::world::grid::{
    BlockSettings, ConverterMode, GeneratorMode, GoalSettings, SignDisplay,
};

pub fn block_json(world: &oif_sim::WorldBlocks, pos: IVec3) -> Value {
    block_json_with_structure(world, None, pos)
}

/// 方块调试信息；可选附带结构摘要
pub fn block_json_with_structure(
    world: &oif_sim::WorldBlocks,
    structures: Option<&oif_sim::simulation::structure_state::StructureState>,
    pos: IVec3,
) -> Value {
    let material_block = world.blocks.get(&pos).copied();
    let system_block = world.system_blocks.get(&pos).copied();
    let machine_body = world.machine_bodies.get(&pos).copied();

    let Some(block) = material_block.or(system_block) else {
        if let Some(body) = machine_body {
            return json!({
                "layer": "machine_body",
                "kind": format!("{:?}", body.kind),
                "kind_detail": kind_detail(body.kind),
                "facing": format!("{:?}", body.facing),
                "yaw": body.facing.yaw(),
                "id": body.id.0,
                "directional": body.kind.is_directional(),
            });
        }
        return Value::Null;
    };

    let layer = if material_block.is_some() {
        block_layer(block.kind)
    } else {
        "system"
    };
    let id = block.id;

    let paints: Vec<Value> = world
        .material_paints
        .iter()
        .filter(|(face, _)| face.block == id)
        .map(|(face, paint)| {
            json!({
                "normal": pos_json(face.normal),
                "paint": paint_string_id(*paint),
            })
        })
        .collect();

    let attached_stamps: Vec<Value> = world
        .material_attachments
        .iter()
        .filter(|(_, att)| att.parent == id)
        .filter_map(|(child_id, att)| {
            let (child_pos, child) = world.blocks.iter().find(|(_, b)| b.id == *child_id)?;
            Some(json!({
                "child_id": child_id.0,
                "pos": pos_json(*child_pos),
                "kind": format!("{:?}", child.kind),
                "stamp": child.kind.stamp_id().map(stamp_string_id),
                "facing": format!("{:?}", child.facing),
                "parent_face_normal": pos_json(att.parent_face_normal),
            }))
        })
        .collect();

    let attachment = world.material_attachments.get(&id).map(|att| {
        let parent_pos = world
            .blocks
            .iter()
            .find(|(_, b)| b.id == att.parent)
            .map(|(p, _)| pos_json(*p));
        json!({
            "parent_id": att.parent.0,
            "parent_pos": parent_pos,
            "parent_face_normal": pos_json(att.parent_face_normal),
        })
    });

    let welds: Vec<Value> = world
        .material_welds
        .iter()
        .filter_map(|weld| {
            let other = weld.other(id)?;
            let other_pos = world
                .blocks
                .iter()
                .find(|(_, b)| b.id == other)
                .map(|(p, _)| pos_json(*p));
            Some(json!({
                "other_id": other.0,
                "other_pos": other_pos,
            }))
        })
        .collect();

    let wire_panels: Vec<Value> = world
        .wire_face_panels
        .iter()
        .filter(|face| face.block == id)
        .map(|face| json!({ "normal": pos_json(face.normal) }))
        .collect();

    let settings = world
        .block_settings
        .get(&pos)
        .map(block_settings_json)
        .unwrap_or(Value::Null);

    let acceptor_id = world.acceptor_id_at(pos).map(|id| id.0);

    let structure_summary = structures.and_then(|state| {
        let id = state.structure_id_at(pos)?;
        let structure = state.get(id)?;
        Some(json!({
            "id": id.0,
            "kind": format!("{:?}", structure.kind),
            "activity": format!("{:?}", structure.activity),
            "pushable": structure.pushable,
            "freedom": format!("{:?}", structure.freedom),
            "movable": structure.pushable && structure.freedom != oif_sim::simulation::structure_state::StructureFreedom::None,
            "size": structure.positions.len(),
        }))
    });

    json!({
        "layer": layer,
        "kind": format!("{:?}", block.kind),
        "kind_detail": kind_detail(block.kind),
        "facing": format!("{:?}", block.facing),
        "yaw": block.facing.yaw(),
        "id": id.0,
        "directional": block.kind.is_directional(),
        "paints": paints,
        "attached_stamps": attached_stamps,
        "attachment": attachment,
        "welds": welds,
        "wire_panels": wire_panels,
        "settings": settings,
        "acceptor_id": acceptor_id,
        "structure": structure_summary,
        "machine_body": machine_body.map(|body| json!({
            "kind": format!("{:?}", body.kind),
            "facing": format!("{:?}", body.facing),
            "id": body.id.0,
        })),
        "system_overlap": material_block.is_some().then(|| system_block.map(|sys| json!({
            "kind": format!("{:?}", sys.kind),
            "facing": format!("{:?}", sys.facing),
            "id": sys.id.0,
        }))).flatten(),
    })
}

/// 结构完整调试信息（成员方块 + 可移动性）
pub fn structure_json(
    world: &oif_sim::WorldBlocks,
    structures: &oif_sim::simulation::structure_state::StructureState,
    id: oif_sim::simulation::structure_state::StructureId,
) -> Option<Value> {
    let structure = structures.get(id)?;
    let mut blocks: Vec<Value> = structure
        .positions
        .iter()
        .map(|pos| {
            let block = world
                .blocks
                .get(pos)
                .or_else(|| world.system_blocks.get(pos));
            json!({
                "pos": pos_json(*pos),
                "kind": block.map(|b| format!("{:?}", b.kind)),
                "id": block.map(|b| b.id.0),
                "facing": block.map(|b| format!("{:?}", b.facing)),
            })
        })
        .collect();
    blocks.sort_by_key(|value| {
        let pos = value.get("pos").cloned().unwrap_or(Value::Null);
        (
            pos.get("x").and_then(|v| v.as_i64()).unwrap_or(0),
            pos.get("y").and_then(|v| v.as_i64()).unwrap_or(0),
            pos.get("z").and_then(|v| v.as_i64()).unwrap_or(0),
        )
    });
    Some(json!({
        "id": id.0,
        "kind": format!("{:?}", structure.kind),
        "activity": format!("{:?}", structure.activity),
        "pushable": structure.pushable,
        "freedom": format!("{:?}", structure.freedom),
        "movable": structure.pushable
            && structure.freedom != oif_sim::simulation::structure_state::StructureFreedom::None,
        "size": structure.positions.len(),
        "blocks": blocks,
        "breakable_splits": structure
            .breakable_splits
            .iter()
            .map(|(block_id, split)| {
                let mut actor: Vec<_> = split.actor_side.iter().copied().collect();
                let mut target: Vec<_> = split.target_side.iter().copied().collect();
                actor.sort_by_key(|p| (p.x, p.y, p.z));
                target.sort_by_key(|p| (p.x, p.y, p.z));
                json!({
                    "block_id": block_id.0,
                    "facing": format!("{:?}", split.facing),
                    "is_bridge": split.is_bridge,
                    "actor_anchored": oif_sim::simulation::structure_state::touches_scene(
                        world,
                        &split.actor_side,
                    ),
                    "target_anchored": oif_sim::simulation::structure_state::touches_scene(
                        world,
                        &split.target_side,
                    ),
                    "actor_side": actor.iter().map(|p| pos_json(*p)).collect::<Vec<_>>(),
                    "target_side": target.iter().map(|p| pos_json(*p)).collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>(),
    }))
}

/// 按坐标 / 方块 ID / 结构 ID 解析结构查询
pub fn resolve_structure_query(
    world: &oif_sim::WorldBlocks,
    structures: &mut oif_sim::simulation::structure_state::StructureState,
    x: Option<i32>,
    y: Option<i32>,
    z: Option<i32>,
    block_id: Option<u64>,
    structure_id: Option<u64>,
) -> Result<Value, String> {
    if structures.is_empty() && !world.blocks.is_empty() {
        structures.rebuild_factory_for_debug(world);
    }

    let id = if let Some(structure_id) = structure_id {
        oif_sim::simulation::structure_state::StructureId(structure_id)
    } else if let Some(block_id) = block_id {
        let pos = world
            .blocks
            .iter()
            .chain(world.system_blocks.iter())
            .find(|(_, block)| block.id.0 == block_id)
            .map(|(pos, _)| *pos)
            .ok_or_else(|| format!("no block with id={block_id}"))?;
        structures.structure_id_at(pos).ok_or_else(|| {
            format!(
                "block id={block_id} at ({}, {}, {}) has no structure",
                pos.x, pos.y, pos.z
            )
        })?
    } else if let (Some(x), Some(y), Some(z)) = (x, y, z) {
        let pos = IVec3::new(x, y, z);
        structures
            .structure_id_at(pos)
            .ok_or_else(|| format!("no structure at ({x}, {y}, {z})"))?
    } else {
        return Err("getStructure requires ?id= or ?blockId= or ?x=&y=&z=".into());
    };

    structure_json(world, structures, id).ok_or_else(|| format!("structure id={} not found", id.0))
}

fn kind_detail(kind: BlockKind) -> Value {
    match kind {
        BlockKind::Material(id) => json!({
            "type": "material",
            "string_id": material_string_id(id),
        }),
        BlockKind::Stamp(id) => json!({
            "type": "stamp",
            "string_id": stamp_string_id(id),
        }),
        BlockKind::Scene(id) => json!({
            "type": "scene",
            "string_id": scene_catalog().string_id(id).unwrap_or("?"),
        }),
        _ => json!({ "type": "block" }),
    }
}

fn material_string_id(id: MaterialBlockId) -> String {
    material_catalog().string_id(id).unwrap_or("?").to_string()
}

fn stamp_string_id(id: StampMaterialId) -> String {
    stamp_catalog().string_id(id).unwrap_or("?").to_string()
}

fn paint_string_id(id: PaintMaterialId) -> String {
    paint_catalog().string_id(id).unwrap_or("?").to_string()
}

fn block_settings_json(settings: &BlockSettings) -> Value {
    match settings {
        BlockSettings::Generator(s) => json!({
            "type": "generator",
            "mode": match s.mode {
                GeneratorMode::Period { period, offset } => json!({
                    "kind": "period",
                    "period": period,
                    "offset": offset,
                }),
                GeneratorMode::Link { anchor } => json!({
                    "kind": "link",
                    "anchor": anchor.map(pos_json),
                }),
            },
            "material": material_string_id(s.material),
            "facing": format!("{:?}", s.facing),
            "yaw": s.facing.yaw(),
        }),
        BlockSettings::Goal(s) => goal_settings_json(s),
        BlockSettings::Stamper(s) => json!({
            "type": "stamper",
            "stamp": stamp_string_id(s.stamp),
        }),
        BlockSettings::Roller(s) => json!({
            "type": "roller",
            "paint": paint_string_id(s.paint),
        }),
        BlockSettings::Converter(s) => json!({
            "type": "converter",
            "mode": match s.mode {
                ConverterMode::AnyInput => "any_input",
                ConverterMode::SpecificInput => "specific_input",
            },
            "input": material_string_id(s.input),
            "output": material_string_id(s.output),
        }),
        BlockSettings::Teleport(s) => json!({
            "type": "teleport",
            "name": s.name,
            "pair": s.pair.map(pos_json),
        }),
        BlockSettings::Sign(s) => json!({
            "type": "sign",
            "text": s.text,
            "display": match s.display {
                Some(SignDisplay::Material(id)) => json!({
                    "kind": "material",
                    "id": material_string_id(id),
                }),
                Some(SignDisplay::Stamp(id)) => json!({
                    "kind": "stamp",
                    "id": stamp_string_id(id),
                }),
                None => Value::Null,
            },
        }),
    }
}

fn goal_settings_json(s: &GoalSettings) -> Value {
    json!({
        "type": "goal",
        "material": material_string_id(s.material),
        "facing": format!("{:?}", s.facing),
        "yaw": s.facing.yaw(),
        "stamps": s.stamps.map(|slot| slot.map(stamp_string_id)),
        "paints": s.paints.map(|slot| slot.map(paint_string_id)),
    })
}

pub fn block_layer(kind: BlockKind) -> &'static str {
    if kind.is_scene() {
        "scene"
    } else if kind.is_factory() {
        "factory"
    } else if kind.is_material() {
        "material"
    } else if kind.is_system_layer() && kind.is_system_block() {
        "system"
    } else {
        "virtual"
    }
}

pub fn pos_json(pos: IVec3) -> Value {
    json!({ "x": pos.x, "y": pos.y, "z": pos.z })
}

pub fn target_hit_json(world: &WorldBlocks, hit: &TargetHit) -> Value {
    json!({
        "pos": pos_json(hit.pos),
        "normal": pos_json(hit.normal),
        "block": block_json(world, hit.pos),
        "place_at": pos_json(hit.pos + hit.normal),
    })
}

pub fn cursor_target_json(placement: &PlacementState, world: &WorldBlocks) -> Value {
    match placement.target.as_ref() {
        Some(hit) => target_hit_json(world, hit),
        None => Value::Null,
    }
}

/// 模拟阶段：未开局 / 连续跑且无动画 / 回合间（含动画或停在回合边界）
pub fn sim_phase_label(simulation: &SimulationState, animating: bool) -> &'static str {
    if !simulation.is_active() {
        "idle"
    } else if animating {
        "between_turns"
    } else if simulation.running {
        "running"
    } else {
        "between_turns"
    }
}

pub fn simulation_status_json(
    simulation: &SimulationState,
    builder_mode: BuilderMode,
    animating: bool,
) -> Value {
    json!({
        "builder_mode": match builder_mode {
            BuilderMode::Edit => "Edit",
            BuilderMode::Play => "Play",
        },
        "active": simulation.is_active(),
        "running": simulation.running,
        "step_requested": simulation.step_requested,
        "turn": simulation.turn,
        "speed": simulation.speed,
        "sim_phase": sim_phase_label(simulation, animating),
        "accumulator": simulation.accumulator,
    })
}

pub fn save_status_json(save_state: &SaveState, solution_state: &SolutionState) -> Value {
    let Some(slot) = save_state.current.as_ref() else {
        return Value::Null;
    };
    let kind = save_state.current_kind.unwrap_or_else(|| slot.kind());
    json!({
        "path": slot.storage_path(),
        "kind": match kind {
            SaveKind::Puzzle => "puzzle",
            SaveKind::Solution => "solution",
        },
        "entry": match solution_state.entry {
            WorldEntryMode::EditPuzzle => "edit_puzzle",
            WorldEntryMode::PlaySolution => "play_solution",
        },
        "puzzle_id": solution_state.puzzle_id,
        "dirty": solution_state.dirty,
    })
}

/// 嵌入式客户端完整 /status（主菜单与 Playing 均可）
pub fn embedded_status_json(
    mode: GameMode,
    builder_mode: BuilderMode,
    playing_ui: &PlayingUiState,
    ui_runtime: &UiRuntime,
    simulation: &SimulationState,
    save_state: &SaveState,
    solution_state: &SolutionState,
    render_ready: bool,
    animating: bool,
    cursor: Value,
) -> Value {
    json!({
        "ok": true,
        "game_mode": match mode {
            GameMode::StartMenu => "start_menu",
            GameMode::Playing => "playing",
        },
        "paused": playing_ui.paused,
        "inventory_open": playing_ui.inventory_open,
        "active_play": playing_ui.active_play(),
        "ui_blocks_gameplay": ui_runtime.blocks_gameplay(),
        "render_ready": render_ready,
        "save": save_status_json(save_state, solution_state),
        "simulation": simulation_status_json(simulation, builder_mode, animating),
        "cursor": cursor,
    })
}

pub fn session_status_json(control: &SimulationControl) -> Value {
    let phase = if !control.is_active() {
        "idle"
    } else if control.running {
        "running"
    } else {
        "between_turns"
    };
    json!({
        "mode": "headless",
        "active": control.is_active(),
        "running": control.running,
        "step_requested": control.step_requested,
        "turn": control.turn,
        "speed": control.speed,
        "sim_phase": phase,
    })
}

pub fn perf_stats_json(
    fps: f64,
    perf: &PerfStats,
    sim_stats: &SimulationStepStats,
    builder_mode: BuilderMode,
    simulation: &SimulationState,
    block_count: usize,
    entity_count: usize,
    player: Option<Vec3>,
) -> Value {
    let render_remainder_ms = (perf.render_other_ms() - perf.render_gap_ms()).max(0.0);
    let scopes: Vec<Value> = PerfScope::ORDER
        .iter()
        .map(|scope| {
            let ms = perf.scope_ms(*scope);
            json!({
                "name": scope.label(),
                "ms": ms,
                "us": ms * 1000.0,
            })
        })
        .collect();
    let sim_turn = (builder_mode == BuilderMode::Play
        && simulation.running
        && sim_stats.has_sample)
        .then(|| {
            json!({
                "total_ms": sim_stats.total_ms,
                "prep_ms": sim_stats.prep_ms,
                "gravity_ms": sim_stats.gravity_ms,
                "signal_ms": sim_stats.signal_ms,
                "marker_before_move_ms": sim_stats.marker_before_move_ms,
                "movement_mark_ms": sim_stats.movement_mark_ms,
                "movement_execute_ms": sim_stats.movement_execute_ms,
                "marker_after_move_ms": sim_stats.marker_after_move_ms,
                "behavior_ms": sim_stats.behavior_ms,
                "signal_refresh_ms": sim_stats.signal_refresh_ms,
                "render_rebuild_ms": sim_stats.render_rebuild_ms,
            })
        });
    json!({
        "fps": fps,
        "frame_ms": perf.frame_ms(),
        "main_ms": perf.main_ms(),
        "main_other_us": perf.main_other_ms() * 1000.0,
        "render_ms": perf.render_other_ms(),
        "render_gap_ms": perf.render_gap_ms(),
        "render_remainder_ms": render_remainder_ms,
        "scopes": scopes,
        "sim_turn": sim_turn,
        "blocks": block_count,
        "entities": entity_count,
        "player": player.map(|pos| json!({
            "x": pos.x,
            "y": pos.y,
            "z": pos.z,
        })),
    })
}

pub fn facing_label(facing: Facing) -> &'static str {
    match facing {
        Facing::North => "N",
        Facing::East => "E",
        Facing::South => "S",
        Facing::West => "W",
    }
}

pub fn target_status_line(placement: &PlacementState, world: &WorldBlocks) -> String {
    let Some(hit) = placement.target.as_ref() else {
        return "Target: —".into();
    };
    let place_at = hit.pos + hit.normal;
    let block_label = world
        .blocks
        .get(&hit.pos)
        .map(|block| format!("{:?}", block.kind))
        .or_else(|| {
            world
                .system_blocks
                .get(&hit.pos)
                .map(|block| format!("{:?}", block.kind))
        })
        .unwrap_or_else(|| "Scene".into());
    let facing = world
        .blocks
        .get(&hit.pos)
        .or_else(|| world.system_blocks.get(&hit.pos))
        .map(|block| facing_label(block.facing))
        .unwrap_or("-");
    format!(
        "Target: ({}, {}, {}) {} [{facing}]   Place: ({}, {}, {})",
        hit.pos.x, hit.pos.y, hit.pos.z, block_label, place_at.x, place_at.y, place_at.z
    )
}
