use bevy::prelude::IVec3;

use super::fixture::{apply_fixture_setup, load_fixture_file, run_fixture_dir, run_fixture_file};
use super::protocol::{help_json, json_error, json_ok, DebugHttpCommand};
use super::snapshot::{block_json, pos_json, session_status_json};
use super::standalone::HeadlessDebugState;
use super::world_ops::{
    block_kinds_json, fixture_root, load_save_into_session, parse_block_kind, parse_facing,
    place_block, reset_session,
};

/// 处理无头 debug HTTP 命令
pub fn handle_headless_command(
    state: &mut HeadlessDebugState,
    command: DebugHttpCommand,
) -> String {
    match command {
        DebugHttpCommand::Help => help_json(),
        DebugHttpCommand::BlockKinds => block_kinds_json(),
        DebugHttpCommand::GetPosBlock { x, y, z } => {
            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                let pos = IVec3::new(x, y, z);
                let world = state.session.world_blocks();
                json_ok(serde_json::json!({
                    "pos": pos_json(pos),
                    "block": block_json(world, pos),
                }))
            } else {
                json_error("headless mode requires ?x=&y=&z= for /getPosBlock")
            }
        }
        DebugHttpCommand::GetStatus => {
            let control = state.session.control();
            json_ok(serde_json::json!({
                "game_mode": "headless",
                "paused": false,
                "inventory_open": false,
                "active_play": true,
                "ui_blocks_gameplay": false,
                "render_ready": true,
                "save": null,
                "simulation": session_status_json(control),
                "cursor": null,
            }))
        }
        DebugHttpCommand::WorldReset => state.with_core(|core| {
            reset_session(core);
            json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
        }),
        DebugHttpCommand::BeginSimulation => state.with_core(|core| {
            core.begin_simulation();
            json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
        }),
        DebugHttpCommand::LoadSave { name } => {
            if name.is_empty() {
                return json_error("loadSave requires ?name=");
            }
            state.with_core(|core| match load_save_into_session(core, &name) {
                Ok(()) => json_ok(serde_json::json!({
                    "save": name,
                    "simulation": session_status_json(core.control()),
                })),
                Err(error) => json_error(&error),
            })
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
        DebugHttpCommand::LoadFixture { path } => {
            if path.is_empty() {
                return json_error("loadFixture requires ?path=");
            }
            match load_fixture_file(&path) {
                Ok(fixture) => {
                    state.with_core(|core| match apply_fixture_setup(core, &fixture) {
                        Ok(()) => json_ok(serde_json::json!({
                            "fixture": fixture.name,
                            "simulation": session_status_json(core.control()),
                        })),
                        Err(error) => json_error(&error),
                    })
                }
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::RunFixture { path } => {
            if path.is_empty() {
                return json_error("runFixture requires ?path=");
            }
            match run_fixture_file_path(state, &path) {
                Ok(fixture) => state.with_core(|core| {
                    json_ok(serde_json::json!({
                        "fixture": fixture.name,
                        "simulation": session_status_json(core.control()),
                    }))
                }),
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::RunAllFixtures => state.with_core(|core| {
            let dir = fixture_root().join("blocks");
            let results = run_fixture_dir(core, &dir);
            let passed = results.iter().filter(|(_, result)| result.is_ok()).count();
            let payload: Vec<_> = results
                .into_iter()
                .map(|(name, result)| {
                    serde_json::json!({
                        "name": name,
                        "ok": result.is_ok(),
                        "error": result.err(),
                    })
                })
                .collect();
            json_ok(serde_json::json!({
                "passed": passed,
                "total": payload.len(),
                "results": payload,
            }))
        }),
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
        DebugHttpCommand::GetPerf => {
            json_error("perf stats only available in the embedded game client")
        }
    }
}

fn run_fixture_file_path(
    state: &mut HeadlessDebugState,
    path: &str,
) -> Result<super::fixture::E2eFixture, String> {
    state.with_core(|core| run_fixture_file(core, path))
}
