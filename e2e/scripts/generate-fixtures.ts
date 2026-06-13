#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "fs";
import { join } from "path";

const ROOT = join(import.meta.dir, "..");
const OUT = join(ROOT, "fixtures/blocks");

const SCENE = new Set(["Grass", "Stone", "Dirt", "Planks"]);
const FACTORY = new Set([
  "Platform",
  "Welder",
  "DownWelder",
  "Conveyor",
  "ReverseConveyor",
  "Detector",
  "DownDetector",
  "Wire",
  "Pusher",
  "Lifter",
  "Rotator",
  "CounterRotator",
  "Blocker",
  "Drill",
  "Laser",
]);
const SYSTEM = new Set([
  "Generator",
  "Goal",
  "Stamper",
  "Roller",
  "Converter",
  "TeleportEntrance",
  "TeleportExit",
]);
const MATERIAL = new Set(["Material", "IronMaterial", "CopperMaterial"]);
const VIRTUAL = new Set(["WeldPoint", "BlockerHead", "DrillHead"]);

const VIRTUAL_PARENT: Record<string, string> = {
  WeldPoint: "Welder",
  BlockerHead: "Blocker",
  DrillHead: "Drill",
};

const ALL = [
  ...SCENE,
  ...FACTORY,
  ...SYSTEM,
  ...MATERIAL,
  ...VIRTUAL,
];

type Place = { x: number; y: number; z: number; kind: string; facing: string };

function setupFor(kind: string): Place[] {
  const stone: Place = { x: 0, y: 0, z: 0, kind: "Stone", facing: "North" };
  if (VIRTUAL.has(kind)) {
    return [stone, { x: 0, y: 1, z: 0, kind: VIRTUAL_PARENT[kind], facing: "North" }];
  }
  if (SCENE.has(kind)) {
    return [{ x: 0, y: 0, z: 0, kind, facing: "North" }];
  }
  if (FACTORY.has(kind)) {
    return [stone, { x: 0, y: 1, z: 0, kind, facing: "North" }];
  }
  if (MATERIAL.has(kind)) {
    return [
      stone,
      { x: 0, y: 1, z: 0, kind: "Platform", facing: "North" },
      { x: 0, y: 2, z: 0, kind, facing: "North" },
    ];
  }
  if (SYSTEM.has(kind)) {
    return [{ x: 0, y: 0, z: 0, kind, facing: "North" }];
  }
  return [stone, { x: 0, y: 1, z: 0, kind, facing: "North" }];
}

function fixtureFor(kind: string) {
  const setup = setupFor(kind);
  if (VIRTUAL.has(kind)) {
    return {
      name: `block_${kind.toLowerCase()}`,
      setup,
      steps: [
        { op: "beginSimulation" },
        { op: "run", turns: 1 },
        {
          op: "assertBlock",
          x: 0,
          y: 1,
          z: -1,
          kind,
          layer: "virtual",
        },
      ],
    };
  }
  const target = setup[setup.length - 1];
  const layer = SCENE.has(kind)
    ? "scene"
    : FACTORY.has(kind)
      ? "factory"
      : MATERIAL.has(kind)
        ? "material"
        : "system";
  return {
    name: `block_${kind.toLowerCase()}`,
    setup,
    steps: [
      {
        op: "assertBlock",
        x: target.x,
        y: target.y,
        z: target.z,
        kind,
        layer,
      },
    ],
  };
}

mkdirSync(OUT, { recursive: true });

for (const kind of ALL) {
  writeFileSync(join(OUT, `${kind}.json`), `${JSON.stringify(fixtureFor(kind), null, 2)}\n`);
}

const simFixtures = join(ROOT, "fixtures/sim");
mkdirSync(simFixtures, { recursive: true });

writeFileSync(
  join(simFixtures, "welder_weld_point.json"),
  `${JSON.stringify(fixtureFor("WeldPoint"), null, 2)}\n`,
);

writeFileSync(
  join(simFixtures, "wire_detector_power.json"),
  `${JSON.stringify(
    {
      name: "wire_detector_power",
      setup: [
        { x: 0, y: 0, z: 0, kind: "Stone", facing: "North" },
        { x: 1, y: 0, z: 0, kind: "Stone", facing: "North" },
        { x: 2, y: 0, z: 0, kind: "Stone", facing: "North" },
        { x: 0, y: 1, z: 0, kind: "Generator", facing: "North" },
        { x: 1, y: 1, z: 0, kind: "Wire", facing: "North" },
        { x: 2, y: 1, z: 0, kind: "Detector", facing: "North" },
      ],
      steps: [
        { op: "beginSimulation" },
        { op: "run", turns: 1 },
        { op: "assertBlock", x: 2, y: 1, z: 0, kind: "Detector", layer: "factory" },
      ],
    },
    null,
    2,
  )}\n`,
);

console.log(`Generated ${ALL.length} block fixtures in ${OUT}`);
