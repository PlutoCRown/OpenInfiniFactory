use bevy::prelude::IVec3;

use super::protocol::{DebugHttpCommand, help_json, json_error, json_ok};
use super::snapshot::{
    acceptors_json, block_json, block_json_with_structure, headless_perf_json, headless_status_json,
    pos_json, power_query_json, resolve_pos_query, resolve_structure_query, session_status_json,
};
use super::standalone::HeadlessDebugState;
use super::world_ops::{
    block_kinds_json, load_save_into_session, parse_block_kind, parse_facing, place_block,
    reset_session,
};

/// 处理无头 debug HTTP 命令
pub fn handle_headless_command(
    state: &mut HeadlessDebugState,
    command: DebugHttpCommand,
) -> String {
    match command {
        DebugHttpCommand::Help => help_json(),
        DebugHttpCommand::BlockKinds => block_kinds_json(),
        DebugHttpCommand::GetPosBlock {
            x,
            y,
            z,
            block_id,
        } => state.with_core(|core| match resolve_pos_query(core.world_blocks(), x, y, z, block_id)
        {
            Ok(pos) => json_ok(serde_json::json!({
                "pos": pos_json(pos),
                "block": block_json_with_structure(
                    core.world_blocks(),
                    Some(&core.structure_state),
                    pos,
                ),
            })),
            Err(error) => json_error(&error),
        }),
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
        DebugHttpCommand::GetPower {
            x,
            y,
            z,
            block_id,
        } => state.with_core(|core| match resolve_pos_query(&core.world, x, y, z, block_id) {
            Ok(pos) => json_ok(power_query_json(
                &mut core.signal_cache,
                &core.world,
                pos,
            )),
            Err(error) => json_error(&error),
        }),
        DebugHttpCommand::GetPlayers => json_ok(serde_json::json!({ "players": [] })),
        DebugHttpCommand::GetAcceptors => state.with_core(|core| {
            json_ok(serde_json::json!({
                "acceptors": acceptors_json(&core.world, &core.structure_state),
            }))
        }),
        DebugHttpCommand::GetStatus => {
            let control = state.session.control();
            headless_status_json(control, state.current_save.as_deref()).to_string()
        }
        DebugHttpCommand::GetPerf => {
            let load_ms = state.last_load_ms;
            state.with_core(|core| json_ok(headless_perf_json(load_ms, &core.stats)))
        }
        DebugHttpCommand::WorldReset => {
            state.current_save = None;
            state.last_load_ms = None;
            state.with_core(|core| {
                reset_session(core);
                json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
            })
        }
        DebugHttpCommand::SessionExit => {
            state.current_save = None;
            state.last_load_ms = None;
            state.with_core(|core| {
                reset_session(core);
                json_ok(serde_json::json!({
                    "exited": true,
                    "simulation": session_status_json(core.control()),
                }))
            })
        }
        DebugHttpCommand::SessionSave => {
            json_error("session/save is not available in headless mode")
        }
        DebugHttpCommand::BeginSimulation => state.with_core(|core| {
            core.begin_simulation();
            json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
        }),
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
            kind,
            facing,
        } => {
            let Some(kind) = parse_block_kind(&kind) else {
                return json_error(&format!("unknown block kind `{kind}`"));
            };
            let Some(facing) = parse_facing(&facing) else {
                return json_error(&format!("unknown facing `{facing}`"));
            };
            state.with_core(|core| {
                let pos = IVec3::new(x, y, z);
                match place_block(core.world_blocks_mut(), pos, kind, facing) {
                    Ok(()) => json_ok(serde_json::json!({
                        "pos": pos_json(pos),
                        "block": block_json(core.world_blocks(), pos),
                    })),
                    Err(error) => json_error(&error),
                }
            })
        }
        DebugHttpCommand::Run => state.with_core(|core| {
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
        }),
        DebugHttpCommand::RunOneTurn => state.with_core(|core| {
            core.begin_simulation();
            core.simulate_next_turn();
            core.log
                .log(core.control().turn, "HTTP /runOneTurn (headless)");
            json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
        }),
        DebugHttpCommand::RunN { n } => state.with_core(|core| {
            core.begin_simulation();
            for _ in 0..n {
                core.simulate_next_turn();
            }
            core.log
                .log(core.control().turn, format!("HTTP /runN n={n}"));
            json_ok(serde_json::json!({
                "simulation": session_status_json(core.control()),
                "turns": n,
            }))
        }),
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
