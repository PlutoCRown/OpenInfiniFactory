#!/usr/bin/env bun
/**
 * Verify exported fixture matches save region, then simulate and dump block positions.
 *
 * Usage:
 *   bun e2e/scripts/debug-pusher-fixture.ts
 *   bun e2e/scripts/debug-pusher-fixture.ts --port 8777
 */
import { DebugClient, spawnDebugServer, waitForServer } from "../src/client";

const port = Number(Bun.argv.find((arg) => arg.startsWith("--port="))?.split("=")[1] ?? 8777);
const FIXTURE = "sim/test_pusher_platform_stuck.json";
const ORIGIN = { x: -14, y: 0, z: -13 };

const WATCH = [
  { label: "stuck platform (world -11,2,-12)", local: { x: 3, y: 2, z: 1 } },
  { label: "pusher (world -12,2,-12)", local: { x: 2, y: 2, z: 1 } },
  { label: "lower platform (world -12,1,-12)", local: { x: 2, y: 1, z: 1 } },
  { label: "top platform (world -14,3,-12)", local: { x: 0, y: 3, z: 1 } },
];

function toWorld(local: { x: number; y: number; z: number }) {
  return {
    x: local.x + ORIGIN.x,
    y: local.y + ORIGIN.y,
    z: local.z + ORIGIN.z,
  };
}

async function blockAt(client: DebugClient, pos: { x: number; y: number; z: number }) {
  const data = await client.get(`/getPosBlock?x=${pos.x}&y=${pos.y}&z=${pos.z}`);
  return data.block ?? null;
}

async function dump(label: string, client: DebugClient, useWorld = false) {
  console.log(`\n=== ${label} ===`);
  for (const item of WATCH) {
    const pos = useWorld ? toWorld(item.local) : item.local;
    const block = await blockAt(client, pos);
    const posLabel = useWorld
      ? `world (${pos.x},${pos.y},${pos.z})`
      : `local (${pos.x},${pos.y},${pos.z})`;
    console.log(`${item.label} @ ${posLabel}:`, block ?? "empty");
  }
}

async function main() {
  const server = spawnDebugServer(port);
  await waitForServer(port);
  const client = new DebugClient(port);

  try {
    console.log("1) Load save solution_3 and sample world coords");
    await client.post("/world/reset");
    let res = await client.post("/loadSave?name=solution_3");
    if (!res.ok) throw new Error(`loadSave failed: ${JSON.stringify(res)}`);
    await dump("save before sim", client, true);

    console.log("\n2) Load exported fixture and sample local coords");
    await client.post("/world/reset");
    res = await client.post(`/loadFixture?path=${FIXTURE}`);
    if (!res.ok) throw new Error(`loadFixture failed: ${JSON.stringify(res)}`);
    await dump("fixture before sim", client, false);

    console.log("\n3) Run 2 simulation turns on fixture");
    await client.post("/beginSimulation");
    res = await client.post("/runN?n=2");
    if (!res.ok) throw new Error(`runN failed: ${JSON.stringify(res)}`);
    await dump("fixture after 2 turns", client, false);

    const logs = await client.get("/logs?limit=80");
    if (logs.ok && Array.isArray(logs.logs) && logs.logs.length > 0) {
      console.log("\n=== recent sim logs ===");
      for (const line of logs.logs.slice(-40)) {
        console.log(line);
      }
    }
  } finally {
    server.kill();
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
