use bevy::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::game::blocks::BlockData;
use crate::game::world::grid::WorldBlocks;
use crate::sim_core::SimCoreWorld;

use super::world_ops::{
    parse_block_kind, parse_facing, place_block, reset_session, resolve_fixture_path,
};

#[derive(Debug, Deserialize)]
pub struct E2eFixture {
    pub name: String,
    #[serde(default)]
    pub setup: Vec<FixturePlace>,
    #[serde(default)]
    pub steps: Vec<FixtureStep>,
}

#[derive(Debug, Deserialize)]
pub struct FixturePlace {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub kind: String,
    #[serde(default = "default_facing")]
    pub facing: String,
}

fn default_facing() -> String {
    "North".into()
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum FixtureStep {
    AssertBlock {
        x: i32,
        y: i32,
        z: i32,
        kind: String,
        #[serde(default)]
        layer: Option<String>,
    },
    AssertBlockAbsent {
        x: i32,
        y: i32,
        z: i32,
    },
    AssertExtendedDeviceCount {
        count: usize,
    },
    Run {
        #[serde(default = "default_turns")]
        turns: u64,
    },
    BeginSimulation,
}

fn default_turns() -> u64 {
    1
}

pub fn load_fixture_file(path: &str) -> Result<E2eFixture, String> {
    let path = resolve_fixture_path(path);
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read fixture {}: {error}", path.display()))?;
    parse_fixture_json(&contents)
}

pub fn parse_fixture_json(json: &str) -> Result<E2eFixture, String> {
    serde_json::from_str(json).map_err(|error| format!("invalid fixture json: {error}"))
}

pub fn apply_fixture_setup(
    core: &mut SimCoreWorld<'_>,
    fixture: &E2eFixture,
) -> Result<(), String> {
    reset_session(core);
    for place in &fixture.setup {
        let kind = parse_block_kind(&place.kind)
            .ok_or_else(|| format!("unknown block kind `{}`", place.kind))?;
        let facing = parse_facing(&place.facing)
            .ok_or_else(|| format!("unknown facing `{}`", place.facing))?;
        place_block(
            &mut core.world_blocks_mut(),
            IVec3::new(place.x, place.y, place.z),
            kind,
            facing,
        )?;
    }
    Ok(())
}

pub fn run_fixture(core: &mut SimCoreWorld<'_>, fixture: &E2eFixture) -> Result<(), String> {
    apply_fixture_setup(core, fixture)?;
    for (index, step) in fixture.steps.iter().enumerate() {
        run_fixture_step(core, step)
            .map_err(|error| format!("{} step {}: {error}", fixture.name, index + 1))?;
    }
    Ok(())
}

pub fn run_fixture_step(core: &mut SimCoreWorld<'_>, step: &FixtureStep) -> Result<(), String> {
    match step {
        FixtureStep::BeginSimulation => {
            core.begin_simulation();
            Ok(())
        }
        FixtureStep::Run { turns } => {
            if !core.is_active() {
                core.begin_simulation();
            }
            for _ in 0..*turns {
                core.simulate_next_turn(
                    crate::game::world::animation::SIMULATION_TURN_SECONDS,
                    None,
                    None,
                );
            }
            Ok(())
        }
        FixtureStep::AssertBlock {
            x,
            y,
            z,
            kind,
            layer,
        } => {
            let pos = IVec3::new(*x, *y, *z);
            let expected =
                parse_block_kind(kind).ok_or_else(|| format!("unknown block kind `{kind}`"))?;
            let actual = block_at(core.world_blocks(), pos)
                .ok_or_else(|| format!("no block at ({x}, {y}, {z})"))?;
            if actual.kind != expected {
                return Err(format!(
                    "expected {expected:?} at ({x}, {y}, {z}), got {:?}",
                    actual.kind
                ));
            }
            if let Some(layer) = layer {
                let actual_layer = super::snapshot::block_layer(actual.kind);
                if !actual_layer.eq_ignore_ascii_case(layer) {
                    return Err(format!(
                        "expected layer `{layer}` at ({x}, {y}, {z}), got `{actual_layer}`"
                    ));
                }
            }
            Ok(())
        }
        FixtureStep::AssertBlockAbsent { x, y, z } => {
            let pos = IVec3::new(*x, *y, *z);
            if block_at(core.world_blocks(), pos).is_some() {
                return Err(format!(
                    "expected no block at ({x}, {y}, {z}), got {:?}",
                    block_at(core.world_blocks(), pos).map(|block| block.kind)
                ));
            }
            Ok(())
        }
        FixtureStep::AssertExtendedDeviceCount { count } => {
            let actual = core.pusher_state().extended_device_positions().len();
            if actual != *count {
                return Err(format!("expected {count} extended devices, got {actual}"));
            }
            Ok(())
        }
    }
}

fn block_at(world: &WorldBlocks, pos: IVec3) -> Option<BlockData> {
    world
        .blocks
        .get(&pos)
        .copied()
        .or_else(|| world.system_blocks.get(&pos).copied())
}

pub fn run_fixture_file(core: &mut SimCoreWorld<'_>, path: &str) -> Result<E2eFixture, String> {
    let fixture = load_fixture_file(path)?;
    run_fixture(core, &fixture)?;
    Ok(fixture)
}

pub fn run_fixture_dir(
    core: &mut SimCoreWorld<'_>,
    dir: &Path,
) -> Vec<(String, Result<(), String>)> {
    let mut results = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|error| error.to_string())
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("?")
            .to_string();
        let result = (|| {
            let fixture = load_fixture_file(path.to_str().unwrap_or_default())?;
            run_fixture(core, &fixture)
        })();
        results.push((name, result));
    }
    results
}
