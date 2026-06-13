//! Import an e2e JSON fixture as paired puzzle + solution saves.
//!
//! Example:
//! ```text
//! cargo run --bin import_fixture -- \
//!   --fixture e2e/fixtures/sim/wire_detector_power.json
//! ```

use bevy::prelude::IVec3;
use open_infinifactory::debug_http::fixture::{load_fixture_file, E2eFixture};
use open_infinifactory::debug_http::world_ops::{parse_block_kind, parse_facing, place_block};
use open_infinifactory::game::blocks::PersistentLayer;
use open_infinifactory::game::state::BuilderMode;
use open_infinifactory::game::ui::InventoryItems;
use open_infinifactory::game::world::grid::WorldBlocks;
use open_infinifactory::shared::save::{save_puzzle, save_solution};
use std::env;
use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("import_fixture: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut fixture_path = None;
    let mut name = None;
    let mut prefix = "e2e".to_string();

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--fixture" | "--path" => {
                fixture_path = Some(next_arg(&args, &mut index)?);
            }
            "--name" => {
                name = Some(next_arg(&args, &mut index)?);
            }
            "--prefix" => {
                prefix = next_arg(&args, &mut index)?;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            flag => return Err(format!("unknown flag `{flag}`")),
        }
        index += 1;
    }

    let fixture_path = fixture_path.ok_or("--fixture path is required")?;
    let fixture = load_fixture_file(&fixture_path)?;
    let base_name = name.unwrap_or_else(|| {
        if fixture.name.is_empty() {
            Path::new(&fixture_path)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("fixture")
                .to_string()
        } else {
            fixture.name.clone()
        }
    });
    if base_name.is_empty() {
        return Err("fixture name is empty".into());
    }

    let puzzle_name = format!("{prefix}_puzzle_{base_name}");
    let solution_name = format!("{prefix}_solution_{base_name}");

    import_fixture_as_saves(&fixture, &puzzle_name, &solution_name)?;

    println!("imported `{fixture_path}`");
    println!("  puzzle:   {puzzle_name}");
    println!("  solution: {solution_name} (puzzle_id={puzzle_name})");
    println!("  setup blocks: {}", fixture.setup.len());
    Ok(())
}

pub fn import_fixture_as_saves(
    fixture: &E2eFixture,
    puzzle_name: &str,
    solution_name: &str,
) -> Result<(), String> {
    let mut full = WorldBlocks::default();
    for place in &fixture.setup {
        let kind = parse_block_kind(&place.kind)
            .ok_or_else(|| format!("unknown block kind `{}`", place.kind))?;
        let facing = parse_facing(&place.facing)
            .ok_or_else(|| format!("unknown facing `{}`", place.facing))?;
        place_block(
            &mut full,
            IVec3::new(place.x, place.y, place.z),
            kind,
            facing,
        )?;
    }

    let mut puzzle_world = WorldBlocks::default();
    for (pos, block) in full.blocks.iter() {
        if block.kind.persistent_layer() == Some(PersistentLayer::Puzzle) {
            puzzle_world.insert(*pos, *block);
        }
    }
    for (pos, block) in full.system_blocks.iter() {
        if block.kind.persistent_layer() == Some(PersistentLayer::Puzzle) {
            puzzle_world.insert(*pos, *block);
        }
    }

    if !save_puzzle(
        &puzzle_world,
        puzzle_name,
        &InventoryItems::for_mode(BuilderMode::Edit),
    ) {
        return Err(format!("failed to write puzzle save `{puzzle_name}`"));
    }
    if !save_solution(
        &full,
        solution_name,
        puzzle_name,
        &InventoryItems::for_mode(BuilderMode::Play),
    ) {
        return Err(format!("failed to write solution save `{solution_name}`"));
    }
    Ok(())
}

fn next_arg(args: &[String], index: &mut usize) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value after `{}`", args[*index - 1]))
}

fn print_help() {
    eprintln!(
        r#"Usage: import_fixture --fixture PATH [options]

Options:
  --path PATH       Alias of --fixture
  --name NAME       Base name (default: fixture JSON `name` or file stem)
  --prefix PREFIX   Save name prefix (default: e2e)
                      writes PREFIX_puzzle_NAME and PREFIX_solution_NAME
"#
    );
}
