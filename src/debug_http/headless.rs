use bevy::prelude::IVec3;

use super::protocol::{DebugHttpCommand, help_json, json_error, json_ok};
use super::snapshot::{
    acceptors_json, block_json_with_structure, blocks_json, headless_perf_json,
    headless_status_json, pos_json, power_query_json, resolve_pos_query, resolve_structure_query,
    session_status_json,
};
use super::standalone::HeadlessDebugState;
use super::world_ops::{
    block_kinds_json, load_save_into_session, parse_block_kind_exact, parse_facing,
    place_blocks_box, reset_session,
};

/// 处理无头 debug HTTP 命令
pub fn handle_headless_command(
    state: &mut HeadlessDebugState,
    command: DebugHttpCommand,
) -> String {
    match command {
        DebugHttpCommand::Help => help_json(),
        DebugHttpCommand::BlockKinds => block_kinds_json(),
        DebugHttpCommand::GetPosBlock { x, y, z, block_id } => state.with_core(|core| {
            match resolve_pos_query(core.world_blocks(), x, y, z, block_id) {
                Ok(pos) => json_ok(serde_json::json!({
                    "pos": pos_json(pos),
                    "block": block_json_with_structure(
                        core.world_blocks(),
                        Some(&core.structure_state),
                        pos,
                    ),
                })),
                Err(error) => json_error(&error),
            }
        }),
        DebugHttpCommand::GetBlocks {
            kind,
            x,
            y,
            z,
            x1,
            y1,
            z1,
            radius,
            limit,
        } => {
            let kind = match kind.as_deref() {
                Some(name) => match parse_block_kind_exact(name) {
                    Some(kind) => Some(kind),
                    None => return json_error(&format!("unknown block kind `{name}`")),
                },
                None => None,
            };
            state.with_core(|core| {
                match blocks_json(
                    core.world_blocks(),
                    Some(&core.structure_state),
                    kind,
                    x,
                    y,
                    z,
                    x1,
                    y1,
                    z1,
                    radius,
                    limit,
                ) {
                    Ok(blocks) => json_ok(blocks),
                    Err(error) => json_error(&error),
                }
            })
        }
        DebugHttpCommand::GetStructure {
            x,
            y,
            z,
            block_id,
            structure_id,
        } => state.with_core(|core| {
            match resolve_structure_query(
                &core.world,
                &mut core.structure_state,
                x,
                y,
                z,
                block_id,
                structure_id,
            ) {
                Ok(structure) => json_ok(serde_json::json!({ "structure": structure })),
                Err(error) => json_error(&error),
            }
        }),
        DebugHttpCommand::GetPower { x, y, z, block_id } => {
            state.with_core(
                |core| match resolve_pos_query(&core.world, x, y, z, block_id) {
                    Ok(pos) => json_ok(power_query_json(&mut core.signal_cache, &core.world, pos)),
                    Err(error) => json_error(&error),
                },
            )
        }
        DebugHttpCommand::GetPlayers => json_ok(serde_json::json!({ "players": [] })),
        DebugHttpCommand::GetAcceptors => state.with_core(|core| {
            json_ok(serde_json::json!({
                "acceptors": acceptors_json(&core.world, &core.structure_state),
            }))
        }),
        DebugHttpCommand::GetStatus => {
            let control = state.session.control();
            headless_status_json(control, state.current_save.as_deref(), state.dirty).to_string()
        }
        DebugHttpCommand::GetPerf => {
            let load_ms = state.last_load_ms;
            state.with_core(|core| json_ok(headless_perf_json(load_ms, &core.stats)))
        }
        DebugHttpCommand::WorldReset => {
            state.current_save = None;
            state.last_load_ms = None;
            state.dirty = false;
            state.with_core(|core| {
                reset_session(core);
                json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
            })
        }
        DebugHttpCommand::SessionExit => {
            state.current_save = None;
            state.last_load_ms = None;
            state.dirty = false;
            state.with_core(|core| {
                reset_session(core);
                json_ok(serde_json::json!({
                    "exited": true,
                    "simulation": session_status_json(core.control()),
                }))
            })
        }
        DebugHttpCommand::SessionSave => {
            json_error("session/save is disabled for ephemeral headless test saves")
        }
        DebugHttpCommand::BeginSimulation => {
            state.dirty = true;
            state.with_core(|core| {
                core.begin_simulation();
                json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
            })
        }
        DebugHttpCommand::SimPause => state.with_core(|core| {
            core.control.running = false;
            json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
        }),
        DebugHttpCommand::LoadSave { name } => {
            if name.is_empty() {
                return json_error("loadSave/session enter requires ?name=");
            }
            match state.with_core(|core| load_save_into_session(core, &name)) {
                Ok(load_ms) => {
                    state.current_save = Some(name.clone());
                    state.last_load_ms = Some(load_ms);
                    state.dirty = false;
                    let control = state.session.control();
                    json_ok(serde_json::json!({
                        "save": name,
                        "load_ms": load_ms,
                        "simulation": session_status_json(control),
                    }))
                }
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::PlaceBlock {
            x,
            y,
            z,
            x1,
            y1,
            z1,
            kind,
            facing,
        } => {
            let Some(kind) = parse_block_kind_exact(&kind) else {
                return json_error(&format!("unknown block kind `{kind}`"));
            };
            let Some(facing) = parse_facing(&facing) else {
                return json_error(&format!("unknown facing `{facing}`"));
            };
            state.dirty = true;
            state.with_core(|core| {
                let a = IVec3::new(x, y, z);
                let b = IVec3::new(x1.unwrap_or(x), y1.unwrap_or(y), z1.unwrap_or(z));
                let (placed, skipped) =
                    place_blocks_box(core.world_blocks_mut(), a, b, kind, facing);
                if placed.is_empty() && !skipped.is_empty() {
                    return json_error(&format!(
                        "cannot place {kind:?} in box ({},{},{})-({},{},{})",
                        a.x, a.y, a.z, b.x, b.y, b.z
                    ));
                }
                let positions_truncated = placed.len() + skipped.len() > 1000;
                json_ok(serde_json::json!({
                    "from": pos_json(a),
                    "to": pos_json(b),
                    "placed": (!positions_truncated).then(|| placed.iter().map(|pos| pos_json(*pos)).collect::<Vec<_>>()),
                    "skipped": (!positions_truncated).then(|| skipped.iter().map(|pos| pos_json(*pos)).collect::<Vec<_>>()),
                    "first_placed": placed.first().map(|pos| pos_json(*pos)),
                    "last_placed": placed.last().map(|pos| pos_json(*pos)),
                    "placed_count": placed.len(),
                    "skipped_count": skipped.len(),
                    "positions_truncated": positions_truncated,
                }))
            })
        }
        DebugHttpCommand::Run => {
            state.dirty = true;
            state.with_core(|core| {
                core.request_continuous_run();
                core.log
                    .log(core.control().turn, "HTTP /run (headless batch)");
                for _ in 0..10 {
                    core.simulate_next_turn();
                }
                json_ok(serde_json::json!({
                    "simulation": session_status_json(core.control()),
                    "note": "headless /run executes 10 turns immediately",
                }))
            })
        }
        DebugHttpCommand::RunOneTurn => {
            state.dirty = true;
            state.with_core(|core| {
                core.begin_simulation();
                core.simulate_next_turn();
                core.log
                    .log(core.control().turn, "HTTP /runOneTurn (headless)");
                json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
            })
        }
        DebugHttpCommand::RunN { n } => {
            state.dirty = true;
            state.with_core(|core| {
                let begin_started = std::time::Instant::now();
                core.begin_simulation();
                let begin_ms = begin_started.elapsed().as_secs_f64() * 1000.0;
                let mut samples = Vec::with_capacity(n.min(100_000) as usize);
                for _ in 0..n {
                    samples.push(core.simulate_next_turn_with_logging(false).stats);
                }
                core.log
                    .log(core.control().turn, format!("HTTP /runN n={n}"));
                let mut total_ms: Vec<f64> = samples.iter().map(|stats| stats.total_ms).collect();
                total_ms.sort_by(f64::total_cmp);
                let sample_count = total_ms.len();
                let percentile = |fraction: f64| {
                    let index = ((sample_count.saturating_sub(1) as f64) * fraction).round() as usize;
                    total_ms[index]
                };
                let divisor = sample_count as f64;
                json_ok(serde_json::json!({
                    "simulation": session_status_json(core.control()),
                    "turns": n,
                    "perf": {
                        "begin_ms": begin_ms,
                        "samples": sample_count,
                        "total_ms": {
                            "mean": total_ms.iter().sum::<f64>() / divisor,
                            "min": total_ms[0],
                            "p50": percentile(0.50),
                            "p95": percentile(0.95),
                            "max": total_ms[sample_count - 1],
                        },
                        "stage_mean_ms": {
                            "prep": samples.iter().map(|stats| stats.prep_ms).sum::<f64>() / divisor,
                            "gravity": samples.iter().map(|stats| stats.gravity_ms).sum::<f64>() / divisor,
                            "signal": samples.iter().map(|stats| stats.signal_ms).sum::<f64>() / divisor,
                            "marker_before_move": samples.iter().map(|stats| stats.marker_before_move_ms).sum::<f64>() / divisor,
                            "movement_mark": samples.iter().map(|stats| stats.movement_mark_ms).sum::<f64>() / divisor,
                            "movement_execute": samples.iter().map(|stats| stats.movement_execute_ms).sum::<f64>() / divisor,
                            "marker_after_move": samples.iter().map(|stats| stats.marker_after_move_ms).sum::<f64>() / divisor,
                            "behavior": samples.iter().map(|stats| stats.behavior_ms).sum::<f64>() / divisor,
                            "signal_refresh": samples.iter().map(|stats| stats.signal_refresh_ms).sum::<f64>() / divisor,
                        },
                    },
                }))
            })
        }
        DebugHttpCommand::GetLogs { limit } => state.session.log.recent_json(limit),
        DebugHttpCommand::ClearLogs => {
            state.session.log.clear();
            r#"{"ok":true}"#.into()
        }
        DebugHttpCommand::TeleportPlayer { .. } => {
            json_error("player teleport only available in the embedded game client")
        }
    }
}
