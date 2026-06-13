#!/usr/bin/env bun
/**
 * Convert between e2e fixtures and game saves.
 *
 * Usage:
 *   bun scripts/e2e_convert.ts fixture-to-save --fixture sim/wire_detector_power.json
 *   bun scripts/e2e_convert.ts save-to-fixture --solution e2e_solution_wire_detector_power --out e2e/fixtures/sim/wire_detector_power.json
 *   bun scripts/e2e_convert.ts import-all
 */
import { existsSync, mkdirSync, readdirSync, unlinkSync } from "fs";
import { join } from "path";

const REPO = join(import.meta.dir, "..");

function runBin(bin: string, binArgs: string[]) {
  return Bun.spawn(["cargo", "run", "--quiet", "--bin", bin, "--", ...binArgs], {
    cwd: REPO,
    stdout: "inherit",
    stderr: "inherit",
  }).exited.then((code) => {
    if (code !== 0) {
      throw new Error(`${bin} failed: ${binArgs.join(" ")}`);
    }
  });
}

function fixtureToSave(fixturePath: string, name?: string) {
  const args = ["--fixture", fixturePath];
  if (name) {
    args.push("--name", name);
  }
  return runBin("import_fixture", args);
}

async function saveToFixture(options: {
  save: string;
  out: string;
  name?: string;
  withSteps?: boolean;
  turns?: number;
}) {
  const args = [
    "--solution",
    options.save,
    "--auto-bounds",
    "--out",
    options.out,
  ];
  if (options.name) {
    args.push("--name", options.name);
  }
  if (options.withSteps) {
    args.push("--with-run-steps");
    if (options.turns != null) {
      args.push("--turns", String(options.turns));
    }
  }
  await runBin("export_fixture", args);
}

function collectFixturePaths(): string[] {
  const dirs = [
    join(REPO, "e2e/fixtures/blocks"),
    join(REPO, "e2e/fixtures/sim"),
  ];
  const paths: string[] = [];
  for (const dir of dirs) {
    if (!existsSync(dir)) {
      continue;
    }
    const subdir = dir.endsWith("/blocks") ? "blocks" : "sim";
    for (const file of new Bun.Glob("*.json").scanSync(dir)) {
      const name = file.split("/").pop() ?? file;
      paths.push(`${subdir}/${name}`);
    }
  }
  paths.sort();
  return paths;
}

async function exportAll() {
  const names = collectE2eSolutionNames();
  const simDir = join(REPO, "e2e/fixtures/sim");
  mkdirSync(simDir, { recursive: true });

  console.log(`Exporting ${names.length} e2e solution saves to e2e/fixtures/sim/ ...`);
  for (const name of names) {
    const out = join(simDir, `${name}.json`);
    console.log(`\n→ e2e_solution_${name} → sim/${name}.json`);
    await saveToFixture({
      save: `e2e_solution_${name}`,
      out,
      name,
    });
  }

  const keep = new Set(names.map((name) => `${name}.json`));
  for (const file of readdirSync(simDir)) {
    if (!file.endsWith(".json") || keep.has(file)) {
      continue;
    }
    const path = join(simDir, file);
    console.log(`\n✕ remove extra fixture ${file}`);
    unlinkSync(path);
  }

  const blocksDir = join(REPO, "e2e/fixtures/blocks");
  if (existsSync(blocksDir)) {
    for (const file of readdirSync(blocksDir)) {
      if (!file.endsWith(".json")) {
        continue;
      }
      console.log(`\n✕ remove block fixture ${file}`);
      unlinkSync(join(blocksDir, file));
    }
  }

  const duplicateRoot = join(REPO, "e2e/e2e");
  if (existsSync(duplicateRoot)) {
    console.log(`\n✕ remove duplicate tree e2e/e2e/`);
    Bun.spawnSync(["rm", "-rf", duplicateRoot]);
  }

  console.log(`\nDone. ${names.length} sim fixtures synced with saves/e2e_solution_*.ron`);
}

function collectE2eSolutionNames(): string[] {
  const savesDir = join(REPO, "saves");
  const names: string[] = [];
  for (const file of readdirSync(savesDir)) {
    if (!file.startsWith("e2e_solution_") || !file.endsWith(".ron")) {
      continue;
    }
    names.push(file.slice("e2e_solution_".length, -".ron".length));
  }
  names.sort();
  return names;
}

async function importAll() {
  const fixtures = collectFixturePaths();
  console.log(`Importing ${fixtures.length} fixtures to saves/e2e_puzzle_* + e2e_solution_* ...`);
  for (const fixture of fixtures) {
    console.log(`\n→ ${fixture}`);
    await fixtureToSave(fixture);
  }
  console.log(`\nDone. Open saves in ${join(REPO, "saves")} (load e2e_solution_* in play mode).`);
}

function usage() {
  console.log(`Usage:
  bun scripts/e2e_convert.ts fixture-to-save --fixture PATH [--name NAME]
  bun scripts/e2e_convert.ts save-to-fixture --solution NAME --out PATH [--name NAME] [--with-steps] [--turns N]
  bun scripts/e2e_convert.ts import-all
  bun scripts/e2e_convert.ts export-all

Examples:
  bun scripts/e2e_convert.ts fixture-to-save --fixture sim/wire_detector_power.json
  bun scripts/e2e_convert.ts save-to-fixture --solution e2e_solution_wire_detector_power --out e2e/fixtures/sim/wire_detector_power.json
  bun scripts/e2e_convert.ts import-all`);
}

function argValue(flag: string): string | undefined {
  const index = Bun.argv.indexOf(flag);
  if (index === -1) {
    return undefined;
  }
  return Bun.argv[index + 1];
}

async function main() {
  const command = Bun.argv[2];
  switch (command) {
    case "fixture-to-save": {
      const fixture = argValue("--fixture");
      if (!fixture) {
        usage();
        process.exit(1);
      }
      await fixtureToSave(fixture, argValue("--name"));
      break;
    }
    case "save-to-fixture": {
      const save = argValue("--solution") ?? argValue("--save");
      const out = argValue("--out");
      if (!save || !out) {
        usage();
        process.exit(1);
      }
      await saveToFixture({
        save,
        out,
        name: argValue("--name"),
        withSteps: Bun.argv.includes("--with-steps"),
        turns: argValue("--turns") ? Number(argValue("--turns")) : undefined,
      });
      break;
    }
    case "import-all":
      await importAll();
      break;
    case "export-all":
      await exportAll();
      break;
    default:
      usage();
      process.exit(command ? 1 : 0);
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
