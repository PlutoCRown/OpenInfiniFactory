use bevy::prelude::*;
use serde_json::{json, Value};

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

pub fn block_json(world: &oif_sim::WorldBlocks, pos: IVec3) -> Value {
    if let Some(block) = world.blocks.get(&pos) {
        json!({
            "layer": block_layer(block.kind),
            "kind": format!("{:?}", block.kind),
            "facing": format!("{:?}", block.facing),
        })
    } else if let Some(block) = world.system_blocks.get(&pos) {
        json!({
            "layer": "system",
            "kind": format!("{:?}", block.kind),
            "facing": format!("{:?}", block.facing),
        })
    } else {
        Value::Null
    }
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
