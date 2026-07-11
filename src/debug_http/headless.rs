use bevy::prelude::*;

use crate::sim_core::{SimulationControl, SimulationDebugLog};

use super::fixture::{apply_fixture_setup, load_fixture_file, run_fixture_dir, run_fixture_file};
use super::protocol::{help_json, json_error, json_ok, DebugHttpCommand};
use super::snapshot::{block_json, pos_json, session_status_json};
use super::standalone::HeadlessDebugState;
use super::world_ops::{
    block_kinds_json, fixture_root, load_save_into_session, parse_block_kind, parse_facing,
    place_block, reset_session,
};

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
                let world = state.app.world();
                json_ok(serde_json::json!({
                    "pos": pos_json(pos),
                    "block": block_json(world.resource(), pos),
                }))
            } else {
                json_error("headless mode requires ?x=&y=&z= for /getPosBlock")
            }
        }
        DebugHttpCommand::GetStatus => {
            let control = state.app.world().resource::<SimulationControl>();
            json_ok(serde_json::json!({
                "simulation": session_status_json(control),
            }))
        }
        DebugHttpCommand::WorldReset => state.with_core(|mut core, _| {
            reset_session(&mut core);
            json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
        }),
        DebugHttpCommand::BeginSimulation => state.with_core(|mut core, _| {
            core.begin_simulation();
            json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
        }),
        DebugHttpCommand::LoadSave { name } => {
            if name.is_empty() {
                return json_error("loadSave requires ?name=");
            }
            state.with_core(
                |mut core, _| match load_save_into_session(&mut core, &name) {
                    Ok(()) => json_ok(serde_json::json!({
                        "save": name,
                        "simulation": session_status_json(core.control()),
                    })),
                    Err(error) => json_error(&error),
                },
            )
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
            state.with_core(|mut core, _| {
                let pos = IVec3::new(x, y, z);
                match place_block(&mut core.world_blocks_mut(), pos, kind, facing) {
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
                    state.with_core(
                        |mut core, _| match apply_fixture_setup(&mut core, &fixture) {
                            Ok(()) => json_ok(serde_json::json!({
                                "fixture": fixture.name,
                                "simulation": session_status_json(core.control()),
                            })),
                            Err(error) => json_error(&error),
                        },
                    )
                }
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::RunFixture { path } => {
            if path.is_empty() {
                return json_error("runFixture requires ?path=");
            }
            match run_fixture_file_path(state, &path) {
                Ok(fixture) => state.with_core(|core, _| {
                    json_ok(serde_json::json!({
                        "fixture": fixture.name,
                        "simulation": session_status_json(core.control()),
                    }))
                }),
                Err(error) => json_error(&error),
            }
        }
        DebugHttpCommand::RunAllFixtures => state.with_core(|mut core, _| {
            let dir = fixture_root().join("blocks");
            let results = run_fixture_dir(&mut core, &dir);
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
        DebugHttpCommand::Run => state.with_core(|mut core, sim_log| {
            core.request_continuous_run();
            sim_log.log(core.control().turn, "HTTP /run (headless batch)");
            for _ in 0..10 {
                core.simulate_next_turn(Some(sim_log), None);
            }
            json_ok(serde_json::json!({
                "simulation": session_status_json(core.control()),
                "note": "headless /run executes 10 turns immediately",
            }))
        }),
        DebugHttpCommand::RunOneTurn => state.with_core(|mut core, sim_log| {
            core.begin_simulation();
            core.simulate_next_turn(Some(sim_log), None);
            sim_log.log(core.control().turn, "HTTP /runOneTurn (headless)");
            json_ok(serde_json::json!({ "simulation": session_status_json(core.control()) }))
        }),
        DebugHttpCommand::RunN { n } => state.with_core(|mut core, sim_log| {
            core.begin_simulation();
            for _ in 0..n {
                core.simulate_next_turn(Some(sim_log), None);
            }
            sim_log.log(core.control().turn, format!("HTTP /runN n={n}"));
            json_ok(serde_json::json!({
                "simulation": session_status_json(core.control()),
                "turns": n,
            }))
        }),
        DebugHttpCommand::GetLogs { limit } => state
            .app
            .world()
            .resource::<SimulationDebugLog>()
            .recent_json(limit),
        DebugHttpCommand::ClearLogs => {
            state
                .app
                .world_mut()
                .resource_mut::<SimulationDebugLog>()
                .clear();
            r#"{"ok":true}"#.into()
        }
    }
}

fn run_fixture_file_path(
    state: &mut HeadlessDebugState,
    path: &str,
) -> Result<super::fixture::E2eFixture, String> {
    state.with_core(|mut core, _| run_fixture_file(&mut core, path))
}
