//! Export a bounding box from a save (puzzle + optional solution) to an e2e JSON fixture.
//!
//! Example:
//! ```text
//! cargo run --bin export_fixture -- \
//!   --solution solution_3 \
//!   --min -14,0,-13 --max -10,3,-11 \
//!   --out e2e/fixtures/sim/test_pusher_platform_stuck.json \
//!   --name test_pusher_platform_stuck
//! ```

use bevy::prelude::IVec3;
use open_infinifactory::debug_http::snapshot::block_layer;
use open_infinifactory::game::blocks::BlockData;
use open_infinifactory::game::world::grid::WorldBlocks;
use open_infinifactory::shared::save::load_world;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
struct ExportedFixture {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<ExportSource>,
    setup: Vec<FixturePlace>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    steps: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct ExportSource {
    save: String,
    min: [i32; 3],
    max: [i32; 3],
    origin: [i32; 3],
}

#[derive(Serialize)]
struct FixturePlace {
    x: i32,
    y: i32,
    z: i32,
    kind: String,
    facing: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("export_fixture: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut save = None;
    let mut min = None;
    let mut max = None;
    let mut out = None;
    let mut name = None;
    let mut normalize = true;
    let mut include_steps = false;
    let mut run_turns = 2u64;
    let mut auto_bounds = false;

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--save" | "--solution" => {
                save = Some(next_arg(&args, &mut index)?);
            }
            "--min" => {
                min = Some(parse_coords(&next_arg(&args, &mut index)?)?);
            }
            "--max" => {
                max = Some(parse_coords(&next_arg(&args, &mut index)?)?);
            }
            "--out" => {
                out = Some(PathBuf::from(next_arg(&args, &mut index)?));
            }
            "--name" => {
                name = Some(next_arg(&args, &mut index)?);
            }
            "--no-normalize" => {
                normalize = false;
            }
            "--with-run-steps" => {
                include_steps = true;
            }
            "--turns" => {
                run_turns = next_arg(&args, &mut index)?
                    .parse()
                    .map_err(|error| format!("invalid turns: {error}"))?;
            }
            "--auto-bounds" => {
                auto_bounds = true;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            flag => return Err(format!("unknown flag `{flag}`")),
        }
        index += 1;
    }

    let save = save.ok_or("--save or --solution is required")?;
    let out = out.ok_or("--out path is required")?;

    let mut world = WorldBlocks::default();
    load_world(&mut world, &save).ok_or_else(|| format!("failed to load save `{save}`"))?;

    let (min_corner, max_corner) = if auto_bounds {
        world_bounds(&world)
    } else {
        let min = min.ok_or("--min x,y,z is required (or use --auto-bounds)")?;
        let max = max.ok_or("--max x,y,z is required (or use --auto-bounds)")?;
        normalize_bounds(min, max)
    };
    let name = name.unwrap_or_else(|| {
        out.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("exported_fixture")
            .to_string()
    });
    let origin = if normalize { min_corner } else { IVec3::ZERO };

    let mut setup = collect_blocks(&world, min_corner, max_corner, origin);
    setup.sort_by(|left, right| {
        layer_rank(&left.kind)
            .cmp(&layer_rank(&right.kind))
            .then_with(|| left.y.cmp(&right.y))
            .then_with(|| left.z.cmp(&right.z))
            .then_with(|| left.x.cmp(&right.x))
    });

    let steps = if include_steps {
        vec![
            serde_json::json!({ "op": "beginSimulation" }),
            serde_json::json!({ "op": "run", "turns": run_turns }),
        ]
    } else {
        Vec::new()
    };

    let fixture = ExportedFixture {
        name: name.clone(),
        source: Some(ExportSource {
            save: save.clone(),
            min: [min_corner.x, min_corner.y, min_corner.z],
            max: [max_corner.x, max_corner.y, max_corner.z],
            origin: [origin.x, origin.y, origin.z],
        }),
        setup,
        steps,
    };

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create dir {}: {error}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&fixture)
        .map_err(|error| format!("serialize fixture: {error}"))?;
    fs::write(&out, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", out.display()))?;

    println!("exported `{}` from `{save}`", out.display());
    println!(
        "region: ({},{},{}) .. ({},{},{})",
        min_corner.x, min_corner.y, min_corner.z, max_corner.x, max_corner.y, max_corner.z
    );
    if normalize {
        println!(
            "normalized origin: ({},{},{})",
            origin.x, origin.y, origin.z
        );
    }
    println!("blocks: {}", fixture.setup.len());
    for place in &fixture.setup {
        println!(
            "  ({},{},{}) {} {:?}",
            place.x, place.y, place.z, place.kind, place.facing
        );
    }
    Ok(())
}

fn next_arg(args: &[String], index: &mut usize) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value after `{}`", args[*index - 1]))
}

fn parse_coords(raw: &str) -> Result<IVec3, String> {
    let parts: Vec<_> = raw.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return Err(format!("expected x,y,z got `{raw}`"));
    }
    Ok(IVec3::new(
        parts[0]
            .parse()
            .map_err(|_| format!("invalid x in `{raw}`"))?,
        parts[1]
            .parse()
            .map_err(|_| format!("invalid y in `{raw}`"))?,
        parts[2]
            .parse()
            .map_err(|_| format!("invalid z in `{raw}`"))?,
    ))
}

fn normalize_bounds(min: IVec3, max: IVec3) -> (IVec3, IVec3) {
    (
        IVec3::new(min.x.min(max.x), min.y.min(max.y), min.z.min(max.z)),
        IVec3::new(min.x.max(max.x), min.y.max(max.y), min.z.max(max.z)),
    )
}

fn world_bounds(world: &WorldBlocks) -> (IVec3, IVec3) {
    let mut min = IVec3::splat(i32::MAX);
    let mut max = IVec3::splat(i32::MIN);
    for pos in world
        .blocks
        .keys()
        .chain(world.system_blocks.keys())
        .copied()
    {
        min = min.min(pos);
        max = max.max(pos);
    }
    if min.x == i32::MAX {
        return (IVec3::ZERO, IVec3::ZERO);
    }
    (min, max)
}

fn in_bounds(pos: IVec3, min: IVec3, max: IVec3) -> bool {
    pos.x >= min.x
        && pos.x <= max.x
        && pos.y >= min.y
        && pos.y <= max.y
        && pos.z >= min.z
        && pos.z <= max.z
}

fn collect_blocks(world: &WorldBlocks, min: IVec3, max: IVec3, origin: IVec3) -> Vec<FixturePlace> {
    let mut places = Vec::new();
    for (pos, block) in world.blocks.iter().chain(world.system_blocks.iter()) {
        if in_bounds(*pos, min, max) {
            places.push(to_place(*pos, *block, origin));
        }
    }
    places
}

fn to_place(pos: IVec3, block: BlockData, origin: IVec3) -> FixturePlace {
    let local = pos - origin;
    FixturePlace {
        x: local.x,
        y: local.y,
        z: local.z,
        kind: format!("{:?}", block.kind),
        facing: format!("{:?}", block.facing),
    }
}

fn layer_rank(kind: &str) -> u8 {
    match block_layer_by_name(kind) {
        "scene" => 0,
        "factory" => 1,
        "system" => 2,
        "material" => 3,
        "virtual" => 4,
        _ => 5,
    }
}

fn block_layer_by_name(kind: &str) -> &'static str {
    open_infinifactory::debug_http::world_ops::parse_block_kind(kind)
        .map(block_layer)
        .unwrap_or("unknown")
}

fn print_help() {
    eprintln!(
        r#"Usage: export_fixture --solution NAME --min x,y,z --max x,y,z --out PATH [options]

Options:
  --save NAME           Alias of --solution
  --name NAME           Fixture name (default: output file stem)
  --no-normalize        Keep world coordinates instead of shifting min corner to origin
  --auto-bounds         Derive min/max from all blocks in the save
  --with-run-steps      Add beginSimulation + run steps to fixture
  --turns N             Turns for --with-run-steps (default: 2)
"#
    );
}
